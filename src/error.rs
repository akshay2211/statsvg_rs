use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("GitHub API error: {0}")]
    GitHub(#[from] reqwest::Error),

    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Missing GH_TOKEN env var")]
    MissingToken,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::MissingToken => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::GitHub(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };
        (status, msg).into_response()
    }
}