/*
   Module `ports` specifies the API by which external modules interact with the website domain.

   All traits are bounded by `Send + Sync + 'static`, since their implementations must be shareable
   between request-handling threads.

   Trait methods are explicitly asynchronous, including `Send` bounds on response types,
   since the application is expected to always run in a multithreaded environment.
*/

use std::future::Future;

use tokio::sync::broadcast::Receiver;

use super::models::website::{
    Contact, ContactEvent, CreateWebsiteError, CreateWebsiteRequest, GeneratedWebsite,
    GeneratedWebsiteEvent, GetWebsitesError, UpdateContactError, UpdateGeneratedWebsiteError,
    Website, WebsiteAiError, WebsiteEvent, WebsiteEventError,
};

/// `WebsiteService` is the public API for the website domain.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait WebsiteService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Website].
    ///
    /// # Errors
    ///
    /// - [CreateWebsiteError::InvalidUrl] if a [Website::new] `source_address` is not a valid url.
    fn create_website(
        &self,
        req: &CreateWebsiteRequest,
    ) -> impl Future<Output = Result<Website, CreateWebsiteError>> + Send;

    /// Get all the [Website], sorted by date.
    fn get_websites(&self) -> impl Future<Output = Result<Vec<Website>, GetWebsitesError>> + Send;
    /// Get a receiver to subscribe to sse
    fn get_receiver(&self) -> Receiver<WebsiteEvent>;
}

/// `WebsiteRepository` represents a store of website data.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait WebsiteRepository: Clone + Send + Sync + 'static {
    /// Asynchronously persist a new [Website].
    ///
    /// # Errors
    ///
    /// - MUST return [CreateWebsiteError::Duplicate] if an [Website] with the same [Website::source_address]
    ///   already exists.
    fn create_website(
        &self,
        req: &CreateWebsiteRequest,
    ) -> impl Future<Output = Result<Website, CreateWebsiteError>> + Send;
    fn get_websites(&self) -> impl Future<Output = Result<Vec<Website>, GetWebsitesError>> + Send;
    fn update_contact(
        &self,
        website_id: i64,
        contact: &Contact,
    ) -> impl Future<Output = Result<(), UpdateContactError>> + Send;

    fn update_generated_website(
        &self,
        website_id: i64,
        generated_website: &GeneratedWebsite,
    ) -> impl Future<Output = Result<(), UpdateGeneratedWebsiteError>> + Send;
}

/// `WebsiteNotifier` triggers notifications for status changes on websites.
///
/// This will be used by the SSE endpoint to notify the browser
///
pub trait WebsiteNotifier: Send + Sync + Clone + 'static {
    fn get_receiver(&self) -> Receiver<WebsiteEvent>;
    fn contact_fetched(
        &self,
        contact: &ContactEvent,
    ) -> impl Future<Output = Result<usize, WebsiteEventError>> + Send;
    fn website_added(
        &self,
        website: &Website,
    ) -> impl Future<Output = Result<usize, WebsiteEventError>> + Send;
    fn website_generated(
        &self,
        generated_website: GeneratedWebsiteEvent,
    ) -> impl Future<Output = Result<usize, WebsiteEventError>> + Send;
}

/// `WebsiteAi` does calls to various AI services like lovable and chagpt for various tasks
pub trait WebsiteAi: Send + Sync + Clone + 'static {
    fn get_contact(
        &self,
        website_source_address: &str,
    ) -> impl Future<Output = Result<Contact, WebsiteAiError>> + Send;
    fn get_full_website(
        &self,
        website_source_address: &str,
    ) -> impl Future<Output = Result<String, WebsiteAiError>> + Send;
    fn generate_new_single_page(
        &self,
        website_source_address: &str,
    ) -> impl Future<Output = Result<GeneratedWebsite, WebsiteAiError>> + Send;
}
