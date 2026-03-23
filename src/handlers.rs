use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::{Html, IntoResponse, Response},
    http::header::CONTENT_TYPE,
    Json, http::StatusCode
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use sqlx::Row;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::state::AppState; 
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use axum::extract::Query;
use tracing::{info, warn, error};

// constantes 
const HISTORY_LIMIT: i64 = 100;
const JWT_EXPIRATION_HOURS: i64 = 24;
const MAX_MESSAGE_LENGTH: usize = 4096;
const USER_COLOR_COUNT: u32 = 8;

// handlers archivos estaticos 
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

// WebSocket 
pub async fn ws_handler(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Result<Response, StatusCode> {
    let token_data = decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    );

    match token_data {
        Ok(data) => {
            let verified_username = data.claims.sub;
            Ok(ws.on_upgrade(move |socket| socket_handle(socket, state, verified_username)))
        },
        Err(_) => {
            warn!("Intento de conexión no autorizada al WebSocket.");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn socket_handle(socket: WebSocket, state: AppState, verified_username: String) {
    let (mut sender, mut receiver) = socket.split();

    // Calcular color deterministicamente del username 
    let color_num = verified_username
        .bytes()
        .fold(0u32, |acc, b| acc + b as u32) % USER_COLOR_COUNT + 1;
    let user_color = format!("user-color-{}", color_num);

    // 1. Cargar historial
    match sqlx::query(
        "SELECT name, msg, color, strftime('%H:%M', datetime(created_at, 'localtime')) as time \
         FROM messages ORDER BY id DESC LIMIT ?"
    )
        .bind(HISTORY_LIMIT)
        .fetch_all(&state.db)
        .await 
    {
        Ok(history) => {
            for row in history.into_iter().rev() {
                let name: String = row.get("name");
                let msg: String = row.get("msg");
                let time: String = row.try_get("time").unwrap_or_default();
                let color: String = row.try_get("color").unwrap_or_else(|_| "user-color-1".to_string());

                let msg_json = json!({ "name": name, "msg": msg, "time": time, "color": color });
                let _ = sender.send(Message::Text(msg_json.to_string().into())).await;
            }
        }
        Err(e) => error!(error = %e, "Error al leer el historial"),
    }

    let mut rx = state.tx.subscribe();

    // 2. Loop de mensajes
    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                if let Ok(msg_text) = msg.to_text() {

                    // Limitar tamaño de mensajes
                    if msg_text.len() > MAX_MESSAGE_LENGTH {
                        continue;
                    }

                    if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(msg_text) {
                        // Forzar identidad y color server-side, ignorar lo que mande el cliente
                        parsed["name"] = json!(verified_username);
                        parsed["color"] = json!(user_color);

                        // Guardar en DB con identidad y color reales
                        if let Some(text) = parsed["msg"].as_str() {
                            let _ = sqlx::query(
                                "INSERT INTO messages (name, msg, color) VALUES (?, ?, ?)"
                            )
                                .bind(&verified_username)
                                .bind(text)
                                .bind(&user_color)
                                .execute(&state.db)
                                .await;
                        }

                        let _ = state.tx.send(parsed.to_string());
                    }
                }
            }
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if sender.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(username = %verified_username, skipped = n, "Receptor lageado");
                    }
                    Err(_) => break,
                }
            }
            else => break,
        }
    }
}

// Types 
#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// Auth 
pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<AuthResponse>)> {
    let username_normalized = payload.username.trim().to_lowercase();

    let hashed_password = hash(&payload.password, DEFAULT_COST).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AuthResponse { message: "Error interno al cifrar la contraseña".into(), token: None })
    ))?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&username_normalized)
        .bind(&hashed_password)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(AuthResponse { message: "Usuario creado con éxito".into(), token: None })
        )),
        Err(_) => Err((
            StatusCode::CONFLICT,
            Json(AuthResponse { message: "El usuario ya existe".into(), token: None })
        )),
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<AuthResponse>)> {
    let username_normalized = payload.username.trim().to_lowercase();

    let user_row = sqlx::query("SELECT password_hash FROM users WHERE username = ?")
        .bind(&username_normalized)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse { message: "Error en DB".into(), token: None })
        ))?;

    if let Some(row) = user_row {
        let stored_hash: String = row.get("password_hash");
        let is_valid = verify(&payload.password, &stored_hash).unwrap_or(false);

        if is_valid {
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(JWT_EXPIRATION_HOURS))
                .expect("Timestamp válido")
                .timestamp() as usize;

            let claims = Claims {
                sub: username_normalized.clone(),
                exp: expiration,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes())
            ).unwrap();

            return Ok((
                StatusCode::OK,
                Json(AuthResponse { 
                    message: "Login exitoso".into(),
                    token: Some(token)
                })
            ));
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(AuthResponse { message: "Credenciales inválidas".into(), token: None })
    ))
}