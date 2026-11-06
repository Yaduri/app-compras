use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

pub fn create_jwt(user_id: Uuid, email: &str, name: &str, secret: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let expiration = now + (30 * 24 * 60 * 60); // 30 dias de validade

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        name: name.to_string(),
        iat: now,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Falha ao gerar JWT: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Erro ao criar hash de senha: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Erro ao verificar hash de senha: {}", e)))
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Cabeçalho de autorização ausente".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Formato de autorização inválido. Use 'Bearer <token>'".to_string(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        let claims = verify_jwt(token, &app_state.config.jwt_secret)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::Unauthorized("ID de usuário inválido contido no token".to_string())
        })?;

        Ok(AuthenticatedUser {
            id: user_id,
            email: claims.email,
            name: claims.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let user_id = Uuid::new_v4();
        let secret = "super-secret-test-key-1234567890123456";
        let token = create_jwt(user_id, "teste@exemplo.com", "Usuário Teste", secret).unwrap();

        let claims = verify_jwt(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, "teste@exemplo.com");
        assert_eq!(claims.name, "Usuário Teste");
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "minhasenhasupersincera123";
        let hash = hash_password(password).unwrap();
        assert_ne!(password, hash);
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("senhaerrada", &hash).unwrap());
    }
}
