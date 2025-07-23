use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;

pub mod create_website;
pub mod get_websites;
pub mod websocket;

/// Represents a response containing an API error and a status code
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

/// Represents an error message for the API
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

/// A successful API response
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (self.0, Json(self.1.0.data)).into_response()
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

/// All the possible errors returned by the API
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    UnprocessableEntity(String),
    InternalServerError(String),
    Unauthorized(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use ApiError::*;

        match self {
            InternalServerError(e) => {
                tracing::error!("{}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponseBody::new_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )),
                )
                    .into_response()
            }
            Unauthorized(e) => {
                tracing::warn!("{}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponseBody::new_error(
                        StatusCode::UNAUTHORIZED,
                        "Unauthorized".to_string(),
                    )),
                )
                    .into_response()
            }
            UnprocessableEntity(e) => {
                tracing::warn!("{}", e);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ApiResponseBody::new_error(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        e.to_string(),
                    )),
                )
                    .into_response()
            }
        }
    }
}
