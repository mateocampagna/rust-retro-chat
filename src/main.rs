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

#[derive(Clone)]
struct AppState{
    // enviar mensajes a clientes
    tx:broadcast::Sender<String>,
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

async fn socket_handle(mut socket:WebSocket, state:AppState){
    // receiver = lo que el cliente ENVIA (cliente -> servidor, browser -> a mi)
    // sender = lo que el servidor ENVIA (servidor -> cliente, yo -> browser)
    let (mut sender, mut receiver) = socket.split();
    let mut rx=state.tx.subscribe();
    loop {
        tokio::select! {
            // op 1: yo (servidor) espero recibir un mensaje del cliente
            Some(Ok(msg)) = receiver.next() => {
                if let Ok(message) = msg.to_text(){
                    let _ = state.tx.send(message.to_string());
                }
            }   
            // op 2: yo (servidor) le envio mensajes al cliente
            Ok(msg) = rx.recv() => {
                if sender.send( Message::Text(msg.into())).await.is_err(){
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
    let (tx, _rx) = broadcast::channel(100);
    let app_state=AppState{tx};
    let app = Router::new()
        .route("/", get(html_handler))
        .route("/chat", get(chat_html_handler))
        .route("/style.css", get(css_handler))
        .route("/client.js", get(js_handler))
        .route("/ws", any(ws_handler)).with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
