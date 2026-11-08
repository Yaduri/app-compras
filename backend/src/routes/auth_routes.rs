use axum::{extract::State, http::HeaderMap, Json};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{create_jwt, hash_password, verify_password, AuthenticatedUser};
use crate::error::AppError;
use crate::memory_db::DEMO_EMAIL;
use crate::rate_limit::client_ip;
use crate::state::AppState;
use crate::validation;
use shared::dto::{AuthResponse, GoogleAuthRequest, LoginRequest, RegisterRequest, UserProfile};

/// Tamanho máximo aceito para um token externo (Supabase/Google)
const MAX_EXTERNAL_TOKEN_LEN: usize = 8192;

fn auth_response(state: &AppState, profile: UserProfile) -> Result<Json<AuthResponse>, AppError> {
    let token = create_jwt(profile.id, &profile.email, &profile.name, &state.config.jwt_secret)?;
    Ok(Json(AuthResponse { token, user: profile }))
}

fn profile_from_row(row: &sqlx::postgres::PgRow) -> Result<UserProfile, AppError> {
    Ok(UserProfile {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        avatar_url: row.try_get("avatar_url")?,
    })
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized("E-mail ou senha incorretos".to_string())
}

fn social_account() -> AppError {
    AppError::Unauthorized(
        "Esta conta foi criada com o Google. Use o botão \"Entrar com Google\".".to_string(),
    )
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    state.limits.auth_ip.check(&format!("ip:{}", client_ip(&headers)))?;

    let name = validation::required_text(&payload.name, "O nome", validation::MAX_USER_NAME)?;
    let email = validation::email(&payload.email)?;
    validation::password(&payload.password)?;

    let password_hash = hash_password(payload.password).await?;

    let profile = if let Some(pool) = &state.pool {
        // A restrição UNIQUE(email) evita duplicidade mesmo com cadastros simultâneos
        let row = sqlx::query(
            r#"
            INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3)
            ON CONFLICT (email) DO NOTHING
            RETURNING id, email, name, avatar_url
            "#,
        )
        .bind(&email)
        .bind(&name)
        .bind(&password_hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Conflict("Já existe uma conta cadastrada com este e-mail".to_string()))?;

        profile_from_row(&row)?
    } else {
        let u = state.memory.create_user(name, email, password_hash, None).await?;
        UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url }
    };

    auth_response(&state, profile)
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || payload.password.is_empty() || email.len() > validation::MAX_EMAIL {
        return Err(invalid_credentials());
    }

    state.limits.auth_ip.check(&format!("ip:{}", client_ip(&headers)))?;
    state.limits.login_email.check(&format!("login:{}", email))?;

    let (profile, stored_hash) = if let Some(pool) = &state.pool {
        let row = sqlx::query(
            "SELECT id, email, name, avatar_url, password_hash FROM users WHERE email = $1",
        )
        .bind(&email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(invalid_credentials)?;

        let hash: Option<String> = row.try_get("password_hash")?;
        (profile_from_row(&row)?, hash)
    } else {
        let (u, hash) = state
            .memory
            .find_user_by_email(&email)
            .await
            .ok_or_else(invalid_credentials)?;
        (
            UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url },
            Some(hash),
        )
    };

    // Conta criada via Google não tem senha (NULL no banco, "" no modo memória)
    let hash = stored_hash.filter(|h| !h.is_empty()).ok_or_else(social_account)?;

    if !verify_password(payload.password, hash).await? {
        return Err(invalid_credentials());
    }

    auth_response(&state, profile)
}

/// Identidade confirmada por um provedor externo
struct ExternalIdentity {
    email: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct SupabaseUserMetadata {
    full_name: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
    picture: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct SupabaseUserResponse {
    email: Option<String>,
    email_confirmed_at: Option<String>,
    user_metadata: Option<SupabaseUserMetadata>,
}

/// 1) Access token emitido pelo Supabase Auth do NOSSO projeto (fluxo OAuth do app)
async fn identity_from_supabase(state: &AppState, token: &str) -> Option<ExternalIdentity> {
    let (url, anon_key) = match (&state.config.supabase_url, &state.config.supabase_anon_key) {
        (Some(u), Some(k)) => (u, k),
        _ => return None,
    };

    let res = state
        .http
        .get(format!("{}/auth/v1/user", url))
        .bearer_auth(token)
        .header("apikey", anon_key)
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }

    let user: SupabaseUserResponse = res.json().await.ok()?;
    // Só aceita e-mails confirmados pelo provedor
    user.email_confirmed_at.as_ref()?;
    let email = user.email?;
    let meta = user.user_metadata;
    Some(ExternalIdentity {
        email,
        name: meta.as_ref().and_then(|m| m.full_name.clone().or_else(|| m.name.clone())),
        avatar_url: meta.as_ref().and_then(|m| m.avatar_url.clone().or_else(|| m.picture.clone())),
    })
}

/// 2) id_token do Google emitido para o NOSSO client ID (confere `aud` e e-mail verificado)
async fn identity_from_google_id_token(state: &AppState, token: &str) -> Option<ExternalIdentity> {
    if state.config.google_client_ids.is_empty() {
        return None;
    }

    let res = state
        .http
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", token)])
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }

    let claims: serde_json::Value = res.json().await.ok()?;
    let str_claim = |key: &str| claims.get(key).and_then(|v| v.as_str()).map(str::to_string);

    let aud = str_claim("aud")?;
    if !state.config.google_client_ids.iter().any(|id| *id == aud) {
        tracing::warn!("id_token do Google com audience não autorizado: {}", aud);
        return None;
    }

    let issuer_ok = matches!(
        str_claim("iss").as_deref(),
        Some("accounts.google.com") | Some("https://accounts.google.com")
    );
    // O tokeninfo devolve email_verified como string "true" (ou booleano em alguns casos)
    let verified = match claims.get("email_verified") {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::String(s)) => s == "true",
        _ => false,
    };
    if !issuer_ok || !verified {
        return None;
    }

    Some(ExternalIdentity {
        email: str_claim("email")?,
        name: str_claim("name"),
        avatar_url: str_claim("picture"),
    })
}

pub async fn google_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    state.limits.auth_ip.check(&format!("ip:{}", client_ip(&headers)))?;

    let token = payload.id_token.trim();
    if token.is_empty() || token.len() > MAX_EXTERNAL_TOKEN_LEN {
        return Err(AppError::BadRequest("Token de login inválido".to_string()));
    }

    let identity = match identity_from_supabase(&state, token).await {
        Some(id) => id,
        None => identity_from_google_id_token(&state, token)
            .await
            .ok_or_else(|| AppError::Unauthorized("Não foi possível validar o login com Google".to_string()))?,
    };

    let email = validation::email(&identity.email)?;
    let name = identity
        .name
        .as_deref()
        .map(validation::truncate_name)
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| validation::truncate_name(email.split('@').next().unwrap_or("Usuário")));

    let profile = if let Some(pool) = &state.pool {
        // Conta existente com o mesmo e-mail (verificado) é reaproveitada.
        // O nome escolhido pelo usuário é mantido; só o avatar é atualizado.
        // A senha é apagada: o cadastro por e-mail não confirma o endereço, então ela pode
        // ter sido definida por outra pessoa antes do dono real entrar com o Google.
        // Se havia senha, as sessões abertas com ela também são revogadas.
        let row = sqlx::query(
            r#"
            INSERT INTO users (email, name, avatar_url) VALUES ($1, $2, $3)
            ON CONFLICT (email) DO UPDATE SET
                avatar_url = COALESCE(EXCLUDED.avatar_url, users.avatar_url),
                tokens_valid_after = CASE
                    WHEN users.password_hash IS NOT NULL THEN NOW()
                    ELSE users.tokens_valid_after
                END,
                password_hash = NULL
            RETURNING id, email, name, avatar_url
            "#,
        )
        .bind(&email)
        .bind(&name)
        .bind(&identity.avatar_url)
        .fetch_one(pool)
        .await?;

        profile_from_row(&row)?
    } else {
        match state.memory.find_user_by_email(&email).await {
            Some((u, _)) => {
                state.memory.clear_password(u.id).await;
                UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url }
            }
            None => {
                let u = state
                    .memory
                    .create_user(name, email, String::new(), identity.avatar_url)
                    .await?;
                UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url }
            }
        }
    };

    auth_response(&state, profile)
}

pub async fn get_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserProfile>, AppError> {
    // Usuário removido mas com token ainda válido: trata como sessão inválida
    let gone = || AppError::Unauthorized("Sessão inválida. Entre novamente.".to_string());

    if let Some(pool) = &state.pool {
        let row = sqlx::query("SELECT id, email, name, avatar_url FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(gone)?;
        Ok(Json(profile_from_row(&row)?))
    } else {
        let u = state.memory.find_user_by_id(user.id).await.ok_or_else(gone)?;
        Ok(Json(UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url }))
    }
}

/// Conta de demonstração compartilhada. Desativada em produção por padrão
/// (ALLOW_DEMO=true reativa), pois todos que a usam veem as mesmas listas.
pub async fn demo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, AppError> {
    if !state.config.allow_demo {
        return Err(AppError::NotFound("Modo demonstração desativado".to_string()));
    }
    state.limits.auth_ip.check(&format!("ip:{}", client_ip(&headers)))?;

    let profile = if let Some(pool) = &state.pool {
        let existing = sqlx::query("SELECT id, email, name, avatar_url FROM users WHERE email = $1")
            .bind(DEMO_EMAIL)
            .fetch_optional(pool)
            .await?;
        match existing {
            Some(r) => profile_from_row(&r)?,
            None => {
                // Senha aleatória: a conta demo só é acessível por esta rota
                let random_password = Uuid::new_v4().to_string();
                let pass_hash = hash_password(random_password).await?;
                let r = sqlx::query(
                    r#"
                    INSERT INTO users (email, name, password_hash) VALUES ($1, 'Usuário Demo', $2)
                    ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
                    RETURNING id, email, name, avatar_url
                    "#,
                )
                .bind(DEMO_EMAIL)
                .bind(pass_hash)
                .fetch_one(pool)
                .await?;
                profile_from_row(&r)?
            }
        }
    } else {
        let (u, _) = state
            .memory
            .find_user_by_email(DEMO_EMAIL)
            .await
            .ok_or_else(|| AppError::NotFound("Usuário demo não encontrado".to_string()))?;
        UserProfile { id: u.id, email: u.email, name: u.name, avatar_url: u.avatar_url }
    };

    auth_response(&state, profile)
}
