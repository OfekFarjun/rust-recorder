use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub enum ApiError {
    CaptureAlreadyInProgress,
    NoCaptureIsRunning,
    InternalServerError(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        ApiError::InternalServerError(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::CaptureAlreadyInProgress => (StatusCode::TOO_EARLY, format!("Capture is already in progress")),
            ApiError::NoCaptureIsRunning => (StatusCode::TOO_EARLY, format!("No capture is running")),
            ApiError::InternalServerError(_msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error, {_msg}")),
        }.into_response()
    }
}