use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// A uniquely identifiable website
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Website {
    pub id: i64,
    pub source_address: String,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ContactEvent {
    pub website_id: i64,
    pub contact: Contact,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Contact {
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
}

impl Website {
    pub fn new(id: i64, source_address: &str) -> Self {
        Self {
            id,
            source_address: source_address.to_string(),
            contact_email: None,
            contact_name: None,
        }
    }
}

/// The fields required by the domain to create an [Website].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateWebsiteRequest {
    pub source_address: Url,
}

impl CreateWebsiteRequest {
    pub fn new(source_address: Url) -> Self {
        Self { source_address }
    }
}

#[derive(Debug, Error)]
pub enum CreateWebsiteError {
    #[error("failed to start transaction")]
    FailedTransaction(sqlx::Error),
    #[error("invalid url {source_address}")]
    InvalidUrl { source_address: String },
    #[error("website with source address {source_address} already exists")]
    Duplicate { source_address: Url },
    #[error("failed to send notification")]
    NotificationFailed,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetWebsitesError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
pub enum WebsiteEvent {
    FetchingContact,
    WebsiteAdded(Website),
    FetchedContact(ContactEvent),
}

#[derive(Debug, Error)]
pub enum WebsiteEventError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WebsiteAiError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    #[error("failed to init openai")]
    FailedToInitOpeanAi,
    #[error("failed to fetch content")]
    FailedToFetchContent,
    #[error("failed to get name and owner from chatgpt")]
    FailedToFetchContact,
}

#[derive(Debug, Error)]
pub enum UpdateContactError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    #[error("failed to start transaction")]
    FailedTransaction(sqlx::Error),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Social {
    instagram: Option<String>,
    facebook: Option<String>,
    google_maps: Option<String>,
    google_reviews: Option<String>,
}
