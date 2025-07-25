use std::sync::Arc;

use crate::domain::website::{
    models::website::{
        ContactEvent, GeneratedWebsiteEvent, Website, WebsiteEvent, WebsiteEventError,
    },
    ports::WebsiteNotifier,
};
use tokio::sync::{
    Mutex,
    broadcast::{Receiver, Sender, channel},
};

#[derive(Clone)]
pub struct EventPublisher {
    tx: Sender<WebsiteEvent>,
    _guard: Arc<Mutex<Receiver<WebsiteEvent>>>,
}

impl Default for EventPublisher {
    fn default() -> Self {
        let buffer_size = 10;
        let (tx, rx) = channel(buffer_size);
        let _guard = Arc::new(Mutex::new(rx));
        Self { tx, _guard }
    }
}
impl WebsiteNotifier for EventPublisher {
    async fn contact_fetched(&self, contact: &ContactEvent) -> Result<usize, WebsiteEventError> {
        tracing::debug!("Sending event for contact_fetched");
        self.tx
            .send(WebsiteEvent::FetchedContact(contact.to_owned()))
            .map_err(|e| {
                tracing::debug!("{}", e);
                WebsiteEventError::Unknown(e.into())
            })
    }

    fn get_receiver(&self) -> Receiver<WebsiteEvent> {
        self.tx.subscribe()
    }

    async fn website_added(&self, website: &Website) -> Result<usize, WebsiteEventError> {
        tracing::debug!("Sending event for website_added");
        self.tx
            .send(WebsiteEvent::WebsiteAdded(website.to_owned()))
            .map_err(|e| {
                tracing::debug!("{}", e);
                WebsiteEventError::Unknown(e.into())
            })
    }

    async fn website_generated(
        &self,
        generated_website: GeneratedWebsiteEvent,
    ) -> Result<usize, WebsiteEventError> {
        tracing::debug!("Sending event for website_generated");
        self.tx
            .send(WebsiteEvent::GeneratedWebsite(generated_website.to_owned()))
            .map_err(|e| {
                tracing::debug!("{}", e);
                WebsiteEventError::Unknown(e.into())
            })
    }
}
