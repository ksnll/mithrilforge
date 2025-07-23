use dotenv::dotenv;
use mithrilforge::{
    config::Config,
    domain::website::service::Service,
    inbound::http::{HttpServer, HttpServerConfig},
    outbound::{ai::Ai, event_publisher::EventPublisher, postgres::Postgres},
};

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let config = Config::from_config()?;
    let pgsql = Postgres::new(&config.database_url).await?;
    let notifier = EventPublisher::default();
    let ai = Ai::default();
    let website_service = Service::new(pgsql, notifier, ai);
    let server_config = HttpServerConfig {
        port: &config.server_port,
        jwks: &config.jwks,
    };
    let http_server = HttpServer::new(website_service, server_config).await?;
    http_server.run().await
}
