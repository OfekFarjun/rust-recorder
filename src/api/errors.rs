use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub enum ApiError {
    CaptureAlreadyInProgress,
    NoCaptureIsRunning,
    InternalServerError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::CaptureAlreadyInProgress => (StatusCode::TOO_EARLY, "Capture is already in progress"),
            ApiError::NoCaptureIsRunning => (StatusCode::TOO_EARLY, "No capture is running"),
            ApiError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        }.into_response()
    }
}