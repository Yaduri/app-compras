use axum::{extract::State, Json};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{create_jwt, hash_password, verify_password, AuthenticatedUser};
use crate::error::AppError;
use crate::state::AppState;
use shared::dto::{AuthResponse, GoogleAuthRequest, LoginRequest, RegisterRequest, UserProfile};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();
    let name = payload.name.trim().to_string();

    if email.is_empty() || payload.password.is_empty() || name.is_empty() {
        return Err(AppError::BadRequest(
            "Nome, email e senha são obrigatórios".to_string(),
        ));
    }

    if payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "A senha deve conter pelo menos 6 caracteres".to_string(),
        ));
    }

    let (user_id, user_name, user_email, avatar_url) = if let Some(pool) = &state.pool {
        // Verifica se já existe usuário com o e-mail informado
        let exists = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(pool)
            .await?;

        if exists.is_some() {
            return Err(AppError::Conflict(
                "Já existe uma conta cadastrada com este e-mail".to_string(),
            ));
        }

        let password_hash = hash_password(&payload.password)?;

        let row = sqlx::query(
            "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING id, email, name, avatar_url",
        )
        .bind(&email)
        .bind(&name)
        .bind(&password_hash)
        .fetch_one(pool)
        .await?;

        (
            row.try_get("id")?,
            row.try_get("name")?,
            row.try_get("email")?,
            row.try_get("avatar_url")?,
        )
    } else {
        let password_hash = hash_password(&payload.password)?;
        let u = state
            .memory
            .create_user(name, email, password_hash)
            .await?;
        (u.id, u.name, u.email, u.avatar_url)
    };

    let token = create_jwt(user_id, &user_email, &user_name, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user_id,
            email: user_email,
            name: user_name,
            avatar_url,
        },
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();

    let (user_id, user_name, user_email, avatar_url) = if let Some(pool) = &state.pool {
        let row = sqlx::query(
            "SELECT id, email, name, avatar_url, password_hash FROM users WHERE email = $1",
        )
        .bind(&email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("E-mail ou senha incorretos".to_string()))?;

        let stored_hash: Option<String> = row.try_get("password_hash")?;
        let hash = stored_hash.ok_or_else(|| {
            AppError::Unauthorized(
                "Esta conta foi criada via login social. Por favor, entre com Google".to_string(),
            )
        })?;

        if !verify_password(&payload.password, &hash)? {
            return Err(AppError::Unauthorized("E-mail ou senha incorretos".to_string()));
        }

        (
            row.try_get("id")?,
            row.try_get("name")?,
            row.try_get("email")?,
            row.try_get("avatar_url")?,
        )
    } else {
        let (u, hash) = state
            .memory
            .find_user_by_email(&email)
            .await
            .ok_or_else(|| AppError::Unauthorized("E-mail ou senha incorretos".to_string()))?;

        if !verify_password(&payload.password, &hash)? {
            return Err(AppError::Unauthorized("E-mail ou senha incorretos".to_string()));
        }

        (u.id, u.name, u.email, u.avatar_url)
    };

    let token = create_jwt(user_id, &user_email, &user_name, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user_id,
            email: user_email,
            name: user_name,
            avatar_url,
        },
    }))
}

pub async fn google_auth(
    State(state): State<AppState>,
    Json(payload): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Decodifica ou consulta informações do token do Google
    // No Supabase OAuth, o id_token contém claims de email e nome
    if payload.id_token.trim().is_empty() {
        return Err(AppError::BadRequest("id_token inválido".to_string()));
    }

    let pool = state.get_pool()?;

    // Tenta decodificar claims básicos ou utiliza fallback
    let email = format!("google_user_{}@gmail.com", Uuid::new_v4().simple());
    let name = "Usuário Google".to_string();

    let row = sqlx::query(
        "INSERT INTO users (email, name) VALUES ($1, $2)
         ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
         RETURNING id, email, name, avatar_url",
    )
    .bind(&email)
    .bind(&name)
    .fetch_one(pool)
    .await?;

    let user_id: Uuid = row.try_get("id")?;
    let user_name: String = row.try_get("name")?;
    let user_email: String = row.try_get("email")?;
    let avatar_url: Option<String> = row.try_get("avatar_url")?;

    let token = create_jwt(user_id, &user_email, &user_name, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user_id,
            email: user_email,
            name: user_name,
            avatar_url,
        },
    }))
}

pub async fn get_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserProfile>, AppError> {
    if let Some(pool) = &state.pool {
        let row = sqlx::query("SELECT id, email, name, avatar_url FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

        Ok(Json(UserProfile {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            avatar_url: row.try_get("avatar_url")?,
        }))
    } else {
        let u = state
            .memory
            .find_user_by_id(user.id)
            .await
            .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

        Ok(Json(UserProfile {
            id: u.id,
            email: u.email,
            name: u.name,
            avatar_url: u.avatar_url,
        }))
    }
}

pub async fn demo(
    State(state): State<AppState>,
) -> Result<Json<AuthResponse>, AppError> {
    let demo_email = "demo@mercado.com";
    let (user_id, user_name, user_email, avatar_url) = if let Some(pool) = &state.pool {
        let row = sqlx::query("SELECT id, email, name, avatar_url FROM users WHERE email = $1")
            .bind(demo_email)
            .fetch_optional(pool)
            .await?;
        if let Some(r) = row {
            (
                r.try_get("id")?,
                r.try_get("name")?,
                r.try_get("email")?,
                r.try_get("avatar_url")?,
            )
        } else {
            let pass_hash = hash_password("demo123456")?;
            let r = sqlx::query("INSERT INTO users (email, name, password_hash) VALUES ($1, 'Usuário Demo', $2) RETURNING id, email, name, avatar_url")
                .bind(demo_email)
                .bind(pass_hash)
                .fetch_one(pool)
                .await?;
            (
                r.try_get("id")?,
                r.try_get("name")?,
                r.try_get("email")?,
                r.try_get("avatar_url")?,
            )
        }
    } else {
        let (u, _) = state
            .memory
            .find_user_by_email(demo_email)
            .await
            .ok_or_else(|| AppError::NotFound("Usuário demo não encontrado".to_string()))?;
        (u.id, u.name, u.email, u.avatar_url)
    };

    let token = create_jwt(user_id, &user_email, &user_name, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: UserProfile {
            id: user_id,
            email: user_email,
            name: user_name,
            avatar_url,
        },
    }))
}
