mod dto;

use std::sync::Arc;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::{Error, Router};
use axum::extract::ws::{Message, WebSocket};
use axum::routing::get;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use validator::Validate;
use dto::MessageDto as MessageDto;

struct AppState {
    tx: tokio::sync::broadcast::Sender<String>,
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    info!("new websocket connection");

    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {

            let message_dto: MessageDto = serde_json::from_str(&msg).unwrap();

            if message_dto.validate().is_err() {
                error!("invalid message: {}", msg);
                continue;
            }

            info!("sending message: {}", message_dto.message);
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                info!("received message: {}", text);
                let _ = tx.send(text.to_string());
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => {
            info!("send task finished");
            send_task.abort()
        },
        _ = &mut recv_task => {
            info!("recv task finished");
            recv_task.abort()
        },
    }
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_failed_upgrade(|error: Error| {
        error!("error upgrading websocket: {}", error.to_string());
    })
        .on_upgrade(|socket| handle_socket(socket, state))
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (tx, _rx) = tokio::sync::broadcast::channel(10);

    let app_state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    info!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
