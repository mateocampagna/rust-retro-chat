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
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use axum::extract::Query;
use jsonwebtoken::{decode, Validation, DecodingKey};

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
pub async fn ws_handler(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>,
    Query(query): Query<WsQuery>, // Exigimos el token en la URL
) -> Result<Response, StatusCode> {
    
    let secret_key = "vaca_mala_super_secreto"; // ¡La misma clave que en el login!

    // Intentamos desencriptar y validar la firma
    let token_data = decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(data) => {
            // ¡Token válido! Extraemos el nombre seguro del JWT
            let verified_username = data.claims.sub;
            
            // Le pasamos el nombre verificado al socket
            Ok(ws.on_upgrade(move |socket| socket_handle(socket, state, verified_username)))
        },
        Err(_) => {
            // Token falso o expirado. ¡Acceso denegado!
            println!("[SECURITY] Intento de conexión no autorizada al WebSocket.");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// Ahora recibe el nombre verificado por el servidor
async fn socket_handle(socket: WebSocket, state: AppState, verified_username: String) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Cargar Historial (esto queda igual)
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
                    if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(msg_text) {
                        
                        // IGNORAMOS el nombre que manda el cliente y FORZAMOS el nombre seguro
                        parsed["name"] = json!(verified_username);
                        
                        // Guardamos en la base de datos con la identidad real
                        if let Some(text) = parsed["msg"].as_str() {
                            let _ = sqlx::query("INSERT INTO messages (name, msg) VALUES (?, ?)")
                                .bind(&verified_username)
                                .bind(text)
                                .execute(&state.db)
                                .await;
                        }
                        
                        // Retransmitimos el JSON corregido y seguro a todos
                        let _ = state.tx.send(parsed.to_string());
                    }
                }
            }   
            Ok(msg) = rx.recv() => {
                if sender.send(Message::Text(msg.into())).await.is_err() { break; }
            }
            else => break,
        }
    }
}
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
    pub token: Option<String>, // Acá va a viajar el JWT
}

// Estructura interna del Token (Los "Datos" de la pulsera)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // El "sujeto" (quién es: mateo)
    exp: usize,  // Cuándo expira (Timestamp)
}

// --- ENDPOINT: REGISTRO ---
pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<AuthResponse>)> {
    let username_normalized = payload.username.trim().to_lowercase();
    // 1. Hashear la contraseña (DEFAULT_COST es 12, un buen balance de seguridad/rendimiento)
    let hashed_password = hash(&payload.password, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse { message: "Error interno al cifrar la contraseña".into() , token:None})
        )
    })?;

    // 2. Intentar guardar en la base de datos
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&username_normalized)
        .bind(&hashed_password)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(AuthResponse { message: "Usuario creado con éxito".into(), token:None })
        )),
        Err(_) => Err((
            // Si falla, lo más probable es que el username ya exista (violación de PRIMARY KEY)
            StatusCode::CONFLICT,
            Json(AuthResponse { message: "El usuario ya existe".into(), token:None })
        )),
    }
}

// --- ENDPOINT: LOGIN ---
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<AuthResponse>)> {
    
    // 1. Normalizamos el usuario a minúsculas
    let username_normalized = payload.username.trim().to_lowercase();

    let user_row = sqlx::query("SELECT password_hash FROM users WHERE username = ?")
        .bind(&username_normalized)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse { message: "Error en DB".into(), token: None }))
        })?;

    if let Some(row) = user_row {
        let stored_hash: String = row.get("password_hash");
        let is_valid = verify(&payload.password, &stored_hash).unwrap_or(false);

        if is_valid {
            // --- MAGIA JWT: Creamos la pulsera ---
            
            // Expira en 24 horas
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("Timestamp válido")
                .timestamp() as usize;

            let claims = Claims {
                sub: username_normalized.clone(),
                exp: expiration,
            };

            // Firmamos el token con una clave secreta (En producción, esto va en un archivo .env)
            let secret_key = "vaca_mala_super_secreto"; // ¡Tu firma única!
            
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret_key.as_ref())
            ).unwrap();

            return Ok((
                StatusCode::OK,
                Json(AuthResponse { 
                    message: "Login exitoso".into(),
                    token: Some(token) // Devolvemos el token al frontend!
                })
            ));
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(AuthResponse { message: "Credenciales inválidas".into(), token: None })
    ))
}