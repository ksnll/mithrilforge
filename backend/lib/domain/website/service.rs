/*!
   Module `service` provides the canonical implementation of the [WebisteService] port. All
   website-domain logic is defined here.
*/

use tokio::sync::broadcast::Receiver;

use super::{
    models::website::{
        ContactEvent, CreateWebsiteError, CreateWebsiteRequest, GetWebsitesError, Website,
        WebsiteEvent,
    },
    ports::{WebsiteAi, WebsiteNotifier, WebsiteRepository, WebsiteService},
};

/// Canonical implementation of the [WebsiteService] port, through which the website domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R, N, A>
where
    R: WebsiteRepository,
    N: WebsiteNotifier,
{
    repository: R,
    notifier: N,
    ai: A,
}

impl<R, N, A> Service<R, N, A>
where
    R: WebsiteRepository,
    N: WebsiteNotifier,
    A: WebsiteAi,
{
    pub fn new(repository: R, notifier: N, ai: A) -> Self {
        Self {
            repository,
            notifier,
            ai,
        }
    }
}

impl<R, N, A> WebsiteService for Service<R, N, A>
where
    R: WebsiteRepository,
    N: WebsiteNotifier,
    A: WebsiteAi,
{
    /// Create the [Website] specified in `req`
    ///
    /// # Errors
    ///
    /// - Propagates any [CreateWebsiteError] returned by the [WebsiteRepository].
    async fn create_website(
        &self,
        req: &CreateWebsiteRequest,
    ) -> Result<Website, CreateWebsiteError> {
        let website = self.repository.create_website(req).await?;
        let ai = self.ai.clone();
        let notifier = self.notifier.clone();
        let repository = self.repository.clone();
        let website_source_address = website.source_address.clone();
        let website_id = website.id;
        notifier
            .website_added(&website)
            .await
            .map_err(|_| CreateWebsiteError::NotificationFailed)?;

        tokio::spawn(async move {
            if let Ok(content) = ai.get_full_website(&website_source_address).await
                && let Ok(contact) = ai.get_contact(&content).await
                && let Ok(_) = repository.update_contact(website_id, &contact).await
                && let Ok(_) = notifier
                    .contact_fetched(&ContactEvent {
                        website_id,
                        contact: contact.clone(),
                    })
                    .await
            // && let Ok(remake) = ai.generate_new_single_page(&content).await
            {
                tracing::debug!("Contact fetched event sent for website {}", website_id);
            } else {
                tracing::error!(
                    "background task for get_contact failed for website: {website_source_address} "
                );
            }
        });
        Ok(website)
    }
    async fn get_websites(&self) -> Result<Vec<Website>, GetWebsitesError> {
        self.repository.get_websites().await
    }

    fn get_receiver(&self) -> Receiver<WebsiteEvent> {
        self.notifier.get_receiver()
    }
}
