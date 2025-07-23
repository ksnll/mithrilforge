/*!
    Module `http` exposes an HTTP server that handles HTTP requests to the application. Its
    implementation is opaque to module consumers.
*/

pub mod extractors;
pub mod handlers;

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use axum::routing::{any, post};
use axum::{Router, routing::get};
use handlers::create_website::create_website;
use handlers::get_websites::get_websites;
use handlers::websocket::websocket;
use http::{
    Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use jwtk::jwk::RemoteJwksVerifier;
use tokio::net;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::domain::website::ports::WebsiteService;

/// Configuration for the HTTP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub port: &'a str,
    pub jwks: &'a str,
}

#[derive(Clone)]
/// The global application state shared between all request handlers.
pub struct AppState<WS: WebsiteService> {
    website_service: Arc<WS>,
    jwt_verifier: Arc<RemoteJwksVerifier>,
}

/// The application's HTTP server. The underlying HTTP package is opaque to module consumers.
pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    /// Returns a new HTTP server bound to the port specified in `config`.
    pub async fn new(
        website_service: impl WebsiteService,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        let static_files = ServeDir::new("static").append_index_html_on_directories(true);
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );
        let jwt_verifier =
            RemoteJwksVerifier::new(config.jwks.to_string(), None, Duration::new(3600, 0));

        let state = AppState {
            website_service: Arc::new(website_service),
            jwt_verifier: Arc::new(jwt_verifier),
        };

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
            .allow_origin(Any);

        let router = axum::Router::new()
            .nest("/api", api_routes())
            .layer(cors)
            .layer(trace_layer)
            .with_state(state)
            .fallback_service(static_files);

        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("failed to listen on {}", config.port))?;

        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("listening on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;
        Ok(())
    }
}

fn api_routes<WS: WebsiteService>() -> Router<AppState<WS>> {
    Router::new()
        .route("/website", post(create_website))
        .route("/websites", get(get_websites))
        .route("/events", any(websocket::<WS>))
}
