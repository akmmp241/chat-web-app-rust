mod dto;
mod handler;
mod libs;
mod service;

use crate::handler::{index_handler, register_handler, room_handler};
use crate::libs::cache::AsyncCache;
use axum::routing::{get, post};
use axum::Router;
use handler::{chat_handler, websocket_handler};
use log::info;
use std::sync::Arc;

pub struct AppState {
    rooms: AsyncCache<tokio::sync::broadcast::Sender<String>>,
    users: AsyncCache<User>,
}

#[derive(Clone, Debug)]
pub struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rooms = AsyncCache::new(60);

    let users = AsyncCache::new(15);

    let app_state = Arc::new(AppState { rooms, users });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/", post(register_handler))
        .route("/chat", get(chat_handler))
        .route("/chat", post(room_handler))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    info!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
