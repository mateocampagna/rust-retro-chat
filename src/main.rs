mod state;    
mod handlers; 

use axum::{routing::{any, get, post}, Router};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::broadcast;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // 1. Conexion DB
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://chat.db?mode=rwc")
        .await
        .expect("No se pudo conectar a SQLite");

    // 2. Tablas
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            msg TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&db).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL
        )"
    ).execute(&db).await.unwrap();

    // 3. Estado y Router
    let (tx, _rx) = broadcast::channel(100);
    let app_state = AppState { tx, db };

    let app = Router::new()
        .route("/", get(handlers::html_handler))
        .route("/chat", get(handlers::chat_html_handler))
        .route("/style.css", get(handlers::css_handler))
        .route("/client.js", get(handlers::js_handler))
        .route("/ws", any(handlers::ws_handler))
        .route("/register", post(handlers::register_handler))
        .route("/login", post(handlers::login_handler))
        .with_state(app_state);

    // 4. Iniciar Servidor
    println!("Servidor corriendo en http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}