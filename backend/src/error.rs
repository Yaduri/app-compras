use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use shared::dto::ApiErrorResponse;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    TooManyRequests(String),
    /// Detalhes ficam só no log; o cliente recebe uma mensagem genérica
    Database(String),
    /// Detalhes ficam só no log; o cliente recebe uma mensagem genérica
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            AppError::Database(detail) => {
                tracing::error!("Erro de banco de dados: {}", detail);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Não foi possível acessar os dados agora. Tente novamente em instantes.".to_string(),
                )
            }
            AppError::Internal(detail) => {
                tracing::error!("Erro interno: {}", detail);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erro interno do servidor. Tente novamente.".to_string(),
                )
            }
        };

        let body = Json(ApiErrorResponse { message });
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if matches!(err, sqlx::Error::RowNotFound) {
            return AppError::NotFound("Registro não encontrado".to_string());
        }

        // Violações de restrição viram erros do cliente em vez de 500
        if let Some(db_err) = err.as_database_error() {
            match db_err.code().as_deref() {
                Some("23505") => return AppError::Conflict("Registro já existe".to_string()),
                Some("23514") | Some("22003") | Some("22P02") => {
                    return AppError::BadRequest("Valor inválido informado".to_string())
                }
                Some("23503") => return AppError::BadRequest("Referência inválida".to_string()),
                _ => {}
            }
        }

        AppError::Database(format!("{:?}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        tracing::debug!("JWT rejeitado: {}", err);
        AppError::Unauthorized("Sessão inválida ou expirada. Entre novamente.".to_string())
    }
}
