use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("script Json invalid: {0}")]
    ScriptInvalid(serde_json::Error),
    #[error("input data Json invalid: {0}")]
    InputDataInvalid(serde_json::Error),
    #[error("{0}")]
    OtherError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AppError::ScriptInvalid(..) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError::OtherError(value)
    }
}
