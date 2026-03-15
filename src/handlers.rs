use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::{Html, IntoResponse, Response},
    http::header::CONTENT_TYPE,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use sqlx::Row;
use crate::state::AppState; 

// handlers de archivos estaticos
pub async fn html_handler() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

pub async fn css_handler() -> impl IntoResponse {
    let style = include_str!("../style.css");
    ([(CONTENT_TYPE, "text/css")], style)
}

pub async fn js_handler() -> &'static str {
    include_str!("../client.js")
}

pub async fn chat_html_handler() -> Html<&'static str> {
    Html(include_str!("../chat.html"))
}

// handler de WebSocket
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| socket_handle(socket, state))
}

async fn socket_handle(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Cargar Historial
    match sqlx::query("SELECT name, msg, strftime('%H:%M', datetime(created_at, 'localtime')) as time FROM messages ORDER BY id DESC LIMIT 100")
        .fetch_all(&state.db)
        .await 
    {
        Ok(history) => {
            for row in history.into_iter().rev() {
                let name: String = row.get("name");
                let msg: String = row.get("msg");
                let time: String = row.try_get("time").unwrap_or_else(|_| "".to_string());
                
                let msg_json = json!({ "name": name, "msg": msg, "time": time });
                let _ = sender.send(Message::Text(msg_json.to_string().into())).await;
            }
        }
        Err(e) => println!("Error al leer el historial: {}", e),
    }

    let mut rx = state.tx.subscribe();

    // 2. Loop de mensajes
    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                if let Ok(msg_text) = msg.to_text() {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg_text) {
                        if let (Some(name), Some(text)) = (parsed["name"].as_str(), parsed["msg"].as_str()) {
                            let _ = sqlx::query("INSERT INTO messages (name, msg) VALUES (?, ?)")
                                .bind(name)
                                .bind(text)
                                .execute(&state.db)
                                .await;
                        }
                    }
                    let _ = state.tx.send(msg_text.to_string());
                }
            }   
            Ok(msg) = rx.recv() => {
                if sender.send(Message::Text(msg.into())).await.is_err() { break; }
            }
            else => break,
        }
    }
}