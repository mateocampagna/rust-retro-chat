use std::clone;
use tokio::sync::broadcast;
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use axum::{
    Router, 
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}}, 
    response::{Html, IntoResponse, Response}, 
    routing::{any,get},
    http::header::CONTENT_TYPE,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};

#[derive(Clone)]
struct AppState{
    // enviar mensajes a clientes
    tx:broadcast::Sender<String>,
    db: Pool<Sqlite>,
}

async fn html_handler() -> Html<&'static str>{
    let res = include_str!("../index.html");
    Html(res)
}
async fn css_handler() -> impl IntoResponse {
    let style = include_str!("../style.css");
    ([(CONTENT_TYPE, "text/css")], style)
}
async fn js_handler() -> &'static str {
    let res=include_str!("../client.js");
    res
}

async fn chat_html_handler() -> Html<&'static str>{
    let res = include_str!("../chat.html");
    Html(res)
}

async fn socket_handle(mut socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    match sqlx::query("SELECT name, msg, strftime('%H:%M', datetime(created_at, 'localtime')) as time FROM messages ORDER BY id DESC LIMIT 100")
        .fetch_all(&state.db)
        .await 
    {
        Ok(history) => {
            for row in history.into_iter().rev() {
                let name: String = row.get("name");
                let msg: String = row.get("msg");
                // Obtenemos la hora. Usamos try_get por si algo falla, no crashee.
                let time: String = row.try_get("time").unwrap_or_else(|_| "".to_string());
                
                let msg_json = json!({
                    "name": name,
                    "msg": msg,
                    "time": time // <-- Agregamos el tiempo al JSON
                });
                
                let _ = sender.send(Message::Text(msg_json.to_string().into())).await;
            }
        }
        Err(e) => {
            println!("⚠️ Error al leer el historial: {}", e);
        }
    }
    

// if let Ok(history) = sqlx::query("SELECT name, msg, strftime('%H:%M', datetime(created_at, 'localtime')) as time FROM messages ORDER BY id DESC LIMIT 100")
//     .fetch_all(&state.db)
//     .await 
// {
//     for row in history.into_iter().rev() {
//         let name: String = row.get("name");
//         let msg: String = row.get("msg");
//         // Obtenemos la hora. Usamos try_get por si algo falla, no crashee.
//         let time: String = row.try_get("time").unwrap_or_else(|_| "".to_string());
        
//         let msg_json = json!({
//             "name": name,
//             "msg": msg,
//             "time": time // <-- Agregamos el tiempo al JSON
//         });
        
//         let _ = sender.send(Message::Text(msg_json.to_string().into())).await;
//     }
// }

    let mut rx = state.tx.subscribe();

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

                            let _ = sqlx::query(
                                "DELETE FROM messages WHERE id NOT IN (
                                    SELECT id FROM messages ORDER BY id DESC LIMIT 100
                                )"
                            )
                            .execute(&state.db)
                            .await;
                        }
                    }

                    let _ = state.tx.send(msg_text.to_string());
                }
            }   
            Ok(msg) = rx.recv() => {
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            else => break,
        }
    }
}

async fn ws_handler(ws:WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| socket_handle(socket, state))
}

#[tokio::main]
async fn main() {
    let db = SqlitePoolOptions::new()
        .max_connections(5).connect("sqlite://chat.db?mode=rwc")
        .await
        .expect("No se pudo conectar a SQLite");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            msg TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&db)
    .await
    .expect("Fallo al crear la tabla");

    let (tx, _rx) = broadcast::channel(100);
    let app_state=AppState{tx, db};
    let app = Router::new()
        .route("/", get(html_handler))
        .route("/chat", get(chat_html_handler))
        .route("/style.css", get(css_handler))
        .route("/client.js", get(js_handler))
        .route("/ws", any(ws_handler)).with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
