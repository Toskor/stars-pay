use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// HTTP-facing error returned by handlers.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    Internal(String),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> &str {
        match self {
            AppError::NotFound(m)
            | AppError::Unauthorized(m)
            | AppError::Forbidden(m)
            | AppError::BadRequest(m)
            | AppError::Internal(m) => m,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status(), self.message())
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if matches!(self, AppError::Internal(_)) {
            tracing::error!(error = %self.message(), "internal error");
        }
        (
            self.status(),
            Json(serde_json::json!({ "error": self.message() })),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("invalid json: {}", err))
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
