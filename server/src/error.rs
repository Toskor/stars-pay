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
    TooManyRequests(String),
    Internal(String),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> &str {
        match self {
            AppError::NotFound(m)
            | AppError::Unauthorized(m)
            | AppError::Forbidden(m)
            | AppError::BadRequest(m)
            | AppError::TooManyRequests(m)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn variants_map_to_http_statuses() {
        let cases = [
            (
                AppError::NotFound("nope".into()),
                StatusCode::NOT_FOUND,
                "nope",
            ),
            (
                AppError::Unauthorized("no token".into()),
                StatusCode::UNAUTHORIZED,
                "no token",
            ),
            (
                AppError::Forbidden("owner only".into()),
                StatusCode::FORBIDDEN,
                "owner only",
            ),
            (
                AppError::BadRequest("bad".into()),
                StatusCode::BAD_REQUEST,
                "bad",
            ),
            (
                AppError::TooManyRequests("slow down".into()),
                StatusCode::TOO_MANY_REQUESTS,
                "slow down",
            ),
            (
                AppError::Internal("boom".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "boom",
            ),
        ];
        for (err, expected_status, expected_msg) in cases {
            let resp = err.into_response();
            assert_eq!(resp.status(), expected_status);
            let body = body_json(resp).await;
            assert_eq!(body["error"], expected_msg);
        }
    }

    #[test]
    fn anyhow_error_becomes_bad_request() {
        let err: AppError = anyhow::anyhow!("oops").into();
        assert!(matches!(err, AppError::BadRequest(ref m) if m == "oops"));
    }

    #[test]
    fn serde_error_becomes_bad_request() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{not json").unwrap_err();
        let err: AppError = serde_err.into();
        assert!(matches!(err, AppError::BadRequest(ref m) if m.starts_with("invalid json")));
    }
}
