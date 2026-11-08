use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

/// Validade do token de sessão
const TOKEN_TTL_SECS: i64 = 30 * 24 * 60 * 60; // 30 dias

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

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        name: name.to_string(),
        iat: now,
        exp: now + TOKEN_TTL_SECS,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Falha ao gerar JWT: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    // Algoritmo fixo: impede troca de algoritmo no cabeçalho do token
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// Valida só a assinatura/validade do token e devolve o ID do usuário.
/// Não confere revogação: para autenticar requisições use `authenticate_token`.
pub fn user_id_from_token(token: &str, secret: &str) -> Option<Uuid> {
    verify_jwt(token, secret)
        .ok()
        .and_then(|claims| Uuid::parse_str(&claims.sub).ok())
}

/// Recusa o token se o usuário não existe mais ou se a sessão foi revogada
/// (`iat` anterior a `users.tokens_valid_after`).
pub async fn ensure_token_current(state: &AppState, user_id: Uuid, issued_at: i64) -> Result<(), AppError> {
    let revoked = || AppError::Unauthorized("Sessão inválida. Entre novamente.".to_string());

    // Comparação em segundos inteiros: o token emitido no mesmo segundo da revogação
    // (o do próprio login que revogou) continua válido.
    let valid_after: i64 = match &state.pool {
        Some(pool) => {
            let row = sqlx::query(
                "SELECT FLOOR(EXTRACT(EPOCH FROM tokens_valid_after))::bigint AS valid_after FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(revoked)?;
            sqlx::Row::try_get(&row, "valid_after")?
        }
        None => state.memory.tokens_valid_after(user_id).await.ok_or_else(revoked)?,
    };

    if issued_at < valid_after {
        return Err(revoked());
    }
    Ok(())
}

/// Validação completa (assinatura, validade e revogação). Usada pelo WebSocket.
pub async fn authenticate_token(state: &AppState, token: &str) -> Option<Uuid> {
    let claims = verify_jwt(token, &state.config.jwt_secret).ok()?;
    let user_id = Uuid::parse_str(&claims.sub).ok()?;
    ensure_token_current(state, user_id, claims.iat).await.ok()?;
    Some(user_id)
}

/// Versão síncrona: use apenas fora do runtime assíncrono (ex.: seed do banco em memória)
pub fn hash_password_blocking(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Erro ao criar hash de senha: {}", e)))
}

/// bcrypt é custoso (~100ms+): roda em thread dedicada para não travar o servidor
pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash_password_blocking(&password))
        .await
        .map_err(|e| AppError::Internal(format!("Tarefa de hash interrompida: {}", e)))?
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|e| AppError::Internal(format!("Tarefa de verificação interrompida: {}", e)))?
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

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Unauthorized("Formato de autorização inválido. Use 'Bearer <token>'".to_string())
        })?;

        let claims = verify_jwt(token.trim(), &app_state.config.jwt_secret)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::Unauthorized("ID de usuário inválido contido no token".to_string())
        })?;

        ensure_token_current(&app_state, user_id, claims.iat).await?;

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
        assert_eq!(user_id_from_token(&token, secret), Some(user_id));
    }

    #[test]
    fn test_jwt_rejects_wrong_secret() {
        let token = create_jwt(Uuid::new_v4(), "a@b.com", "A", "segredo-correto-0123456789-0123456789").unwrap();
        assert!(verify_jwt(&token, "segredo-errado-0123456789-0123456789x").is_err());
    }

    #[tokio::test]
    async fn test_tokens_revoked_when_google_takes_over_password_account() {
        let config = std::sync::Arc::new(crate::config::Config {
            port: 0,
            database_url: None,
            jwt_secret: "segredo-de-teste-0123456789-0123456789".to_string(),
            supabase_url: None,
            supabase_anon_key: None,
            google_client_ids: vec![],
            is_production: false,
            allow_demo: false,
            run_migrations: false,
            migrations_dir: String::new(),
        });
        let state = AppState::new(config, None, std::sync::Arc::new(crate::hub::BroadcastHub::new()));
        let user = state
            .memory
            .create_user("A".into(), "a@b.com".into(), "hash".into(), None)
            .await
            .unwrap();

        let old_iat = Utc::now().timestamp() - 60;
        assert!(ensure_token_current(&state, user.id, old_iat).await.is_ok());

        state.memory.clear_password(user.id).await;
        assert!(ensure_token_current(&state, user.id, old_iat).await.is_err());
        // O token emitido agora (pelo próprio login com Google) continua aceito
        assert!(ensure_token_current(&state, user.id, Utc::now().timestamp()).await.is_ok());
        // Usuário inexistente
        assert!(ensure_token_current(&state, Uuid::new_v4(), old_iat).await.is_err());
    }

    #[tokio::test]
    async fn test_password_hashing_and_verification() {
        let password = "minhasenhasupersincera123".to_string();
        let hash = hash_password(password.clone()).await.unwrap();
        assert_ne!(password, hash);
        assert!(verify_password(password, hash.clone()).await.unwrap());
        assert!(!verify_password("senhaerrada".to_string(), hash).await.unwrap());
    }
}
