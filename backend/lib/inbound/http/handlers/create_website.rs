use axum::{Json, extract::State};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{
    domain::website::{
        models::website::{CreateWebsiteError, CreateWebsiteRequest, Website},
        ports::WebsiteService,
    },
    inbound::http::{AppState, extractors::Jwt},
};

use super::{ApiError, ApiSuccess};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateWebsiteResponseData {
    id: String,
}

impl From<CreateWebsiteError> for ApiError {
    fn from(e: CreateWebsiteError) -> Self {
        match e {
            CreateWebsiteError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
            CreateWebsiteError::InvalidUrl { source_address } => {
                tracing::error!("Url is not valid {}\n", source_address);
                Self::UnprocessableEntity(format!("Url is not valid {source_address}",))
            }
            CreateWebsiteError::Duplicate { source_address } => {
                tracing::error!("Url is duplicated {}\n", source_address);
                Self::UnprocessableEntity(format!("Url is not valid {source_address}"))
            }
            CreateWebsiteError::FailedTransaction(cause) => {
                tracing::error!("{:?}", cause);
                Self::InternalServerError("Internal server error".to_string())
            }
            CreateWebsiteError::NotificationFailed => {
                tracing::error!("Failed notification");
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<&Website> for CreateWebsiteResponseData {
    fn from(website: &Website) -> Self {
        Self {
            id: website.id.to_string(),
        }
    }
}

/// The body of an [Website] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateWebsiteHttpRequestBody {
    source_address: String,
}

#[derive(Debug, Clone, Error)]
enum ParseCreateWebsiteHttpRequestError {
    #[error(transparent)]
    SourceAddress(#[from] url::ParseError),
}

impl From<ParseCreateWebsiteHttpRequestError> for ApiError {
    fn from(e: ParseCreateWebsiteHttpRequestError) -> Self {
        let message = match e {
            ParseCreateWebsiteHttpRequestError::SourceAddress(parse_error) => {
                format!("cannot parse url: {parse_error}")
            }
        };
        Self::UnprocessableEntity(message)
    }
}

impl CreateWebsiteHttpRequestBody {
    /// converts the HTTP request body into a domain request. We could use serde as well to avoid
    /// some boilerplate, in case this is infallible
    fn try_into_domain(&self) -> Result<CreateWebsiteRequest, ParseCreateWebsiteHttpRequestError> {
        let url = Url::parse(&self.source_address)?;
        Ok(CreateWebsiteRequest::new(url))
    }
}

pub async fn create_website<WS: WebsiteService>(
    Jwt { user_id, .. }: Jwt<WS>,
    State(state): State<AppState<WS>>,
    Json(body): Json<CreateWebsiteHttpRequestBody>,
) -> Result<ApiSuccess<CreateWebsiteResponseData>, ApiError> {
    tracing::debug!("Decoded user {user_id}");
    let create_website_request = body.try_into_domain()?;
    state
        .website_service
        .create_website(&create_website_request)
        .await
        .map_err(ApiError::from)
        .map(|ref website| ApiSuccess::new(StatusCode::CREATED, website.into()))
}
