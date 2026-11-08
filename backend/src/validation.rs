//! Validação e normalização das entradas recebidas pela API

use crate::error::AppError;

pub const MAX_USER_NAME: usize = 80;
pub const MAX_EMAIL: usize = 254;
pub const MIN_PASSWORD: usize = 6;
/// bcrypt ignora tudo após 72 bytes
pub const MAX_PASSWORD_BYTES: usize = 72;
pub const MAX_LIST_NAME: usize = 80;
pub const MAX_LIST_DESCRIPTION: usize = 300;
pub const MAX_ITEM_NAME: usize = 120;
pub const MAX_NOTES: usize = 500;
pub const MAX_INVITE_CODE: usize = 16;
pub const MAX_QUANTITY: f64 = 100_000.0;
/// Limite de NUMERIC(10, 2)
pub const MAX_PRICE: f64 = 99_999_999.99;

fn too_long(field: &str, max: usize) -> AppError {
    AppError::BadRequest(format!("{} deve ter no máximo {} caracteres", field, max))
}

/// Texto obrigatório: remove espaços nas pontas e rejeita vazio ou longo demais
pub fn required_text(value: &str, field: &str, max: usize) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{} é obrigatório", field)));
    }
    if trimmed.chars().count() > max {
        return Err(too_long(field, max));
    }
    Ok(trimmed.to_string())
}

/// Texto opcional: string vazia vira `None`
pub fn optional_text(value: Option<&str>, field: &str, max: usize) -> Result<Option<String>, AppError> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(v) if v.chars().count() > max => Err(too_long(field, max)),
        Some(v) => Ok(Some(v.to_string())),
    }
}

/// Nome de usuário vindo de provedor externo: corta em vez de rejeitar
pub fn truncate_name(value: &str) -> String {
    let trimmed = value.trim();
    trimmed.chars().take(MAX_USER_NAME).collect()
}

pub fn email(value: &str) -> Result<String, AppError> {
    let email = value.trim().to_lowercase();
    let valid = email.len() <= MAX_EMAIL
        && !email.contains(char::is_whitespace)
        && email
            .split_once('@')
            .map(|(user, domain)| !user.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
            .unwrap_or(false);
    if !valid {
        return Err(AppError::BadRequest("Informe um e-mail válido".to_string()));
    }
    Ok(email)
}

pub fn password(value: &str) -> Result<(), AppError> {
    if value.chars().count() < MIN_PASSWORD {
        return Err(AppError::BadRequest(format!(
            "A senha deve conter pelo menos {} caracteres",
            MIN_PASSWORD
        )));
    }
    if value.len() > MAX_PASSWORD_BYTES {
        return Err(AppError::BadRequest("A senha é longa demais".to_string()));
    }
    Ok(())
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn quantity(value: f64) -> Result<f64, AppError> {
    let rounded = round2(value);
    if !value.is_finite() || rounded <= 0.0 || rounded > MAX_QUANTITY {
        return Err(AppError::BadRequest(
            "A quantidade deve ser um número maior que zero".to_string(),
        ));
    }
    Ok(rounded)
}

pub fn price(value: Option<f64>) -> Result<Option<f64>, AppError> {
    match value {
        None => Ok(None),
        Some(v) if !v.is_finite() || v < 0.0 || v > MAX_PRICE => {
            Err(AppError::BadRequest("Preço inválido".to_string()))
        }
        Some(v) => Ok(Some(round2(v))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_text() {
        assert_eq!(required_text("  Leite ", "Nome", 10).unwrap(), "Leite");
        assert!(required_text("   ", "Nome", 10).is_err());
        assert!(required_text("abcdefghijk", "Nome", 10).is_err());
        assert_eq!(optional_text(Some("  "), "Desc", 10).unwrap(), None);
    }

    #[test]
    fn validates_numbers() {
        assert_eq!(quantity(1.234).unwrap(), 1.23);
        assert!(quantity(0.0).is_err());
        assert!(quantity(-1.0).is_err());
        assert!(quantity(f64::NAN).is_err());
        assert!(price(Some(-0.01)).is_err());
        assert!(price(Some(f64::INFINITY)).is_err());
        assert_eq!(price(Some(10.005)).unwrap(), Some(10.01));
        assert_eq!(price(None).unwrap(), None);
    }

    #[test]
    fn validates_email() {
        assert_eq!(email(" Ana@Mail.com ").unwrap(), "ana@mail.com");
        assert!(email("ana").is_err());
        assert!(email("ana@mail").is_err());
        assert!(email("a b@mail.com").is_err());
    }
}
