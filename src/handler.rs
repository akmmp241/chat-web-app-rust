use crate::service::{create_room, handle_register};
use crate::{dto, service::handle_socket, AppState};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::{header, HeaderMap};
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Error, Json};
use dto::RegisterDto;
use log::{error, info};
use serde::Deserialize;
use std::string::String;
use std::sync::Arc;

const COOKIE_TOKEN: &str = "token";

#[derive(Deserialize, Debug)]
pub struct WsParams {
    pub room: String,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> impl IntoResponse {
    ws.on_failed_upgrade(|error: Error| {
        error!("error upgrading websocket: {}", error.to_string());
    })
    .on_upgrade(move |socket| {
        info!("websocket upgraded, params: {:?}", params);
        handle_socket(socket, state, params.room)
    })
}

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsParams>,
) -> impl IntoResponse {
    let state_clone = Arc::clone(&state);

    let query = params.room;

    let room = state_clone.rooms.get(query.as_str()).await;
    if room.is_none() {
        return Redirect::to("/").into_response();
    }

    Html(include_str!("chat.html")).into_response()
}

pub async fn room_handler(
    State(state): State<Arc<AppState>>,
    Json(data): Json<WsParams>,
) -> impl IntoResponse {
    let res = create_room(state, &data).await;

    res.into_response()
}

pub async fn index_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Html<&'static str> {
    if let Some(cookie) = headers.get(header::COOKIE) {
        let cookie_str = cookie.to_str().unwrap();

        for cookie in cookie_str.split("; ") {
            let mut split = cookie.split("=");
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                if key == COOKIE_TOKEN {
                    let user = state.users.get(value).await;
                    if let Some(_) = user {
                        return Html(include_str!("room.html"));
                    }
                }
            }
        }
    }

    Html(include_str!("index.html"))
}

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterDto>,
) -> impl IntoResponse {
    let (status_code, result) = handle_register(state, &payload).await;

    let res = result.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        format!(
            "token={}; HttpOnly; Path=/; Secure; SameSite=Strict; Max-Age={}",
            res.token,
            5 * 60
        )
        .parse()
        .unwrap(),
    );

    (status_code, headers, Json(res))
}
