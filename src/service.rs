use crate::dto::{RegisterDto, RegisterResponse, RoomResponse};
use crate::handler::WsParams;
use crate::{dto::MessageDto, libs, AppState, User};
use anyhow::anyhow;
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use validator::Validate;

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>, query: String) {
    info!("new websocket connection");

    let (mut sender, mut receiver) = socket.split();

    let room: Sender<String> = {
        let check_room = state.rooms.get(query.as_str()).await;
        if let Some(room) = check_room {
            room.clone()
        } else {
            let (s, _r) = tokio::sync::broadcast::channel(10);
            let _ = state.rooms.set(query, s.clone(), 15 * 60).await;
            s
        }
    };

    let mut rx = room.subscribe();

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

    let tx = room.clone();
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

pub async fn create_room(state: Arc<AppState>, data: &WsParams) -> impl IntoResponse {
    let state_clone = Arc::clone(&state);

    let res: RoomResponse = RoomResponse { room: data.room.clone() };
    let body = serde_json::to_string(&res).unwrap();

    let room = state_clone.rooms.get(data.room.as_str()).await;
    if room.is_some() {
        return Response::builder()
            .status(StatusCode::CREATED)
            .body(body)
            .unwrap();
    }

    let (s, _r) = tokio::sync::broadcast::channel(10);
    state.rooms.set(data.room.clone(), s, 15 * 60).await;

    Response::builder()
        .status(StatusCode::CREATED)
        .body(body)
        .unwrap()
}

pub async fn handle_register(
    state: Arc<AppState>,
    payload: &RegisterDto,
) -> (StatusCode, anyhow::Result<RegisterResponse>) {
    {
        let user = state.users.get(payload.username.as_str()).await;

        if let Some(_) = user {
            return (StatusCode::CONFLICT, Err(anyhow!("username already taken")));
        }
    }

    let user = User {
        username: payload.username.clone(),
        password: payload.password.clone(),
    };

    let token = libs::token::generate_token(6);

    let _ = state.users.set(token.clone(), user, 5 * 60).await;

    let res = RegisterResponse {
        token,
        redirect_url: "".into(),
    };

    (StatusCode::CREATED, Ok(res))
}
