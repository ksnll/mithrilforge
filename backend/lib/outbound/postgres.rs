use anyhow::Context;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::str::FromStr;

use crate::domain::website::{
    models::website::{
        Contact, CreateWebsiteError, CreateWebsiteRequest, GetWebsitesError, UpdateContactError,
        Website,
    },
    ports::WebsiteRepository,
};

#[derive(Debug, Clone)]
pub struct Postgres {
    pool: PgPool,
}
impl Postgres {
    pub async fn new(path: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPool::connect_with(
            PgConnectOptions::from_str(path)
                .with_context(|| format!("invalid database path {path}"))?,
        )
        .await
        .with_context(|| format!("failed to open database at {path}"))?;

        Ok(Self { pool })
    }
}

impl WebsiteRepository for Postgres {
    async fn create_website(
        &self,
        req: &CreateWebsiteRequest,
    ) -> Result<Website, CreateWebsiteError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(CreateWebsiteError::FailedTransaction)?;
        let inserted_website = sqlx::query!(
            r#"WITH inserted_website AS (INSERT INTO websites(source_address) VALUES ($1) RETURNING website_id, source_address)
            SELECT inserted_website.website_id, inserted_website.source_address FROM inserted_website"#,
            req.source_address.to_string()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| CreateWebsiteError::Unknown(e.into()))?;
        tx.commit()
            .await
            .map_err(|e| CreateWebsiteError::Unknown(e.into()))?;
        let website = Website::new(
            inserted_website.website_id,
            &inserted_website.source_address,
        );
        Ok(website)
    }

    async fn get_websites(&self) -> Result<Vec<Website>, GetWebsitesError> {
        let websites = sqlx::query_as!(
            Website,
            r#"
            SELECT website_id as id, source_address, contact_name, contact_email FROM websites"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GetWebsitesError::Unknown(e.into()))?;
        Ok(websites)
    }

    async fn update_contact(
        &self,
        website_id: i64,
        contact: &Contact,
    ) -> Result<(), UpdateContactError> {
        tracing::debug!("Updating contact for {}", website_id);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(UpdateContactError::FailedTransaction)?;
        sqlx::query!(
            r#"UPDATE websites SET contact_email = $1, contact_name = $2 WHERE website_id = $3"#,
            contact.contact_email,
            contact.contact_name,
            website_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| UpdateContactError::Unknown(e.into()))?;
        tx.commit()
            .await
            .map_err(|e| UpdateContactError::Unknown(e.into()))?;
        Ok(())
    }
}
