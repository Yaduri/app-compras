use std::sync::Arc;

/// Tamanho mínimo aceito para o segredo de assinatura dos JWTs
const MIN_JWT_SECRET_LEN: usize = 32;

/// Segredo usado apenas em desenvolvimento local (nunca em produção)
const DEV_JWT_SECRET: &str = "dev-only-jwt-secret-nao-use-em-producao-0123456789";

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: Option<String>,
    pub jwt_secret: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    /// Client IDs OAuth do Google aceitos como `aud` de um id_token
    /// (GOOGLE_CLIENT_ID pode conter vários, separados por vírgula: web, Android...)
    pub google_client_ids: Vec<String>,
    /// Produção: exige banco e segredo JWT, desativa o fallback em memória
    pub is_production: bool,
    /// Habilita a rota /api/auth/demo (padrão: só fora de produção)
    pub allow_demo: bool,
    /// Executa as migrações da pasta `migrations/` ao iniciar
    pub run_migrations: bool,
    pub migrations_dir: String,
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn env_bool(key: &str) -> Option<bool> {
    env_non_empty(key).map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "sim"))
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        // Railway define RAILWAY_ENVIRONMENT automaticamente em seus deploys
        let is_production = env_non_empty("APP_ENV")
            .map(|v| v.eq_ignore_ascii_case("production") || v.eq_ignore_ascii_case("producao"))
            .unwrap_or(false)
            || env_non_empty("RAILWAY_ENVIRONMENT").is_some();

        let port = env_non_empty("PORT")
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url = env_non_empty("DATABASE_URL");
        if is_production && database_url.is_none() {
            return Err("DATABASE_URL é obrigatória em produção".to_string());
        }

        let jwt_secret = match env_non_empty("SUPABASE_JWT_SECRET").or_else(|| env_non_empty("JWT_SECRET")) {
            Some(secret) if secret.len() >= MIN_JWT_SECRET_LEN => secret,
            Some(_) => {
                return Err(format!(
                    "O segredo JWT (SUPABASE_JWT_SECRET/JWT_SECRET) precisa ter pelo menos {} caracteres",
                    MIN_JWT_SECRET_LEN
                ))
            }
            None if is_production => {
                return Err("SUPABASE_JWT_SECRET (ou JWT_SECRET) é obrigatório em produção".to_string())
            }
            None => {
                tracing::warn!("Nenhum segredo JWT configurado: usando segredo de DESENVOLVIMENTO");
                DEV_JWT_SECRET.to_string()
            }
        };

        let supabase_url = env_non_empty("SUPABASE_URL").map(|u| u.trim_end_matches('/').to_string());
        let supabase_anon_key = env_non_empty("SUPABASE_ANON_KEY");

        let google_client_ids = env_non_empty("GOOGLE_CLIENT_ID")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if supabase_url.is_none() || supabase_anon_key.is_none() {
            tracing::warn!("SUPABASE_URL/SUPABASE_ANON_KEY ausentes: login via Supabase desativado");
        }

        let allow_demo = env_bool("ALLOW_DEMO").unwrap_or(!is_production);
        let run_migrations = env_bool("RUN_MIGRATIONS").unwrap_or(false);
        let migrations_dir = env_non_empty("MIGRATIONS_DIR").unwrap_or_else(|| "migrations".to_string());

        Ok(Self {
            port,
            database_url,
            jwt_secret,
            supabase_url,
            supabase_anon_key,
            google_client_ids,
            is_production,
            allow_demo,
            run_migrations,
            migrations_dir,
        })
    }
}

pub type SharedConfig = Arc<Config>;
