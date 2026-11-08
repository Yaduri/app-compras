use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;

use crate::error::AppError;

/// Limitador simples de janela fixa, em memória (suficiente para uma instância)
pub struct RateLimiter {
    max_hits: u32,
    window: Duration,
    entries: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    pub fn new(max_hits: u32, window: Duration) -> Self {
        Self {
            max_hits,
            window,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str) -> Result<(), AppError> {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        // Limpeza periódica para não crescer indefinidamente
        if entries.len() > 10_000 {
            let window = self.window;
            entries.retain(|_, (_, started)| now.duration_since(*started) < window);
        }

        let entry = entries.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) >= self.window {
            *entry = (0, now);
        }
        entry.0 += 1;

        if entry.0 > self.max_hits {
            Err(AppError::TooManyRequests(
                "Muitas tentativas em pouco tempo. Aguarde alguns minutos e tente novamente.".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

pub struct RateLimiters {
    /// Login por e-mail
    pub login_email: RateLimiter,
    /// Login, cadastro e Google por IP
    pub auth_ip: RateLimiter,
    /// Entrada em listas por código (evita adivinhar códigos)
    pub join: RateLimiter,
}

impl RateLimiters {
    pub fn new() -> Self {
        Self {
            login_email: RateLimiter::new(10, Duration::from_secs(15 * 60)),
            auth_ip: RateLimiter::new(60, Duration::from_secs(15 * 60)),
            join: RateLimiter::new(20, Duration::from_secs(10 * 60)),
        }
    }
}

impl Default for RateLimiters {
    fn default() -> Self {
        Self::new()
    }
}

/// IP do cliente. Atrás do proxy do Railway, vem no X-Forwarded-For.
/// Usa o ÚLTIMO valor, que é o acrescentado pelo proxy: os anteriores vêm do próprio
/// cliente e podem ser inventados para burlar o limite de tentativas.
pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get_all("x-forwarded-for")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .next_back()
        .and_then(|v| v.rsplit(',').next())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "desconhecido".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_err());
        assert!(limiter.check("b").is_ok());
    }

    #[test]
    fn client_ip_uses_proxy_appended_value() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.1.1.1, 2.2.2.2, 9.9.9.9".parse().unwrap());
        assert_eq!(client_ip(&headers), "9.9.9.9");
        headers.append("x-forwarded-for", "8.8.8.8".parse().unwrap());
        assert_eq!(client_ip(&headers), "8.8.8.8");
        assert_eq!(client_ip(&HeaderMap::new()), "desconhecido");
    }
}
