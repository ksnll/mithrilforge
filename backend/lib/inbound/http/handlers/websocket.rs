use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use serde::Serialize;

use crate::{
    domain::website::ports::WebsiteService,
    inbound::http::{AppState, extractors::QueryJwt},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetWebsiteResponseData {
    websites: Vec<WebsiteResponseData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WebsiteResponseData {
    id: i64,
}

pub async fn websocket<WS: WebsiteService>(
    ws: WebSocketUpgrade,
    // and still 15 years later header authentication is not a thing in websockets.
    // Jwt { user_id, .. }: Jwt<WS>,
    QueryJwt { user_id, .. }: QueryJwt<WS>,
    State(state): State<AppState<WS>>,
) -> Response {
    tracing::debug!("Decoded user {user_id}");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
async fn handle_socket<WS: WebsiteService>(mut socket: WebSocket, state: AppState<WS>) {
    let mut rx = state.website_service.get_receiver();

    while let Ok(message) = rx.recv().await {
        let event_data = serde_json::to_string(&message).unwrap_or("".to_string());
        if socket.send(Message::Text(event_data.into())).await.is_err() {
            return;
        }
    }
}
