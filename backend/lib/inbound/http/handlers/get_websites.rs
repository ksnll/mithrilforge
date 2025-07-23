use axum::extract::State;
use http::StatusCode;
use serde::Serialize;

use crate::{
    domain::website::{
        models::website::{GetWebsitesError, Website},
        ports::WebsiteService,
    },
    inbound::http::{AppState, extractors::Jwt},
};

use super::{ApiError, ApiSuccess};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetWebsiteResponseData {
    websites: Vec<WebsiteResponseData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WebsiteResponseData {
    id: i64,
    source_address: String,
    contact_email: Option<String>,
    contact_name: Option<String>,
}

impl From<GetWebsitesError> for ApiError {
    fn from(e: GetWebsitesError) -> Self {
        match e {
            GetWebsitesError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<&Vec<Website>> for GetWebsiteResponseData {
    fn from(websites: &Vec<Website>) -> Self {
        Self {
            websites: websites
                .iter()
                .map(|website| WebsiteResponseData {
                    id: website.id,
                    source_address: website.source_address.clone(),
                    contact_email: website.contact_email.clone(),
                    contact_name: website.contact_name.clone(),
                })
                .collect(),
        }
    }
}

pub async fn get_websites<WS: WebsiteService>(
    Jwt { user_id, .. }: Jwt<WS>,
    State(state): State<AppState<WS>>,
) -> Result<ApiSuccess<GetWebsiteResponseData>, ApiError> {
    tracing::debug!("Decoded user {user_id}");
    state
        .website_service
        .get_websites()
        .await
        .map_err(ApiError::from)
        .map(|ref websites| ApiSuccess::new(StatusCode::OK, websites.into()))
}
