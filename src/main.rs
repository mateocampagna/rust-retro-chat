mod state;    
mod handlers; 

use axum::{routing::{any, get, post}, Router};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::broadcast;
use std::net::SocketAddr;
use std::time::Duration;
use crate::state::AppState;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const BROADCAST_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET no está definido en el entorno");

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://chat.db?mode=rwc")
        .await
        .expect("No se pudo conectar a SQLite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            msg TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'user-color-1',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&db).await.expect("No se pudo crear la tabla messages");

    sqlx::query(
        "ALTER TABLE messages ADD COLUMN color TEXT NOT NULL DEFAULT 'user-color-1'"
    ).execute(&db).await.ok();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL
        )"
    ).execute(&db).await.expect("No se pudo crear la tabla users");

    // Rate limiter: 5 intentos rápidos, luego 1 intento cada 30 segundos por IP
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(30)  // 1 token cada 30 segundos
            .burst_size(5)   // 5 intentos rápidos al inicio
            .finish()
            .expect("Configuracion de rate limiter invalida"),
    );

    // Tarea en background para limpiar IPs inactivas de memoria cada 60s
    let governor_limiter = governor_conf.limiter().clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        governor_limiter.retain_recent();
    });

    // Box::leak convierte el Box en una referencia &'static — necesario porque
    // GovernorLayer requiere que el config viva para siempre (dura toda la app)
    let governor_layer = GovernorLayer::new(governor_conf);

    let (tx, _rx) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
    let app_state = AppState { tx, db, jwt_secret };

    let auth_routes = Router::new()
        .route("/register", post(handlers::register_handler))
        .route("/login", post(handlers::login_handler))
        .layer(governor_layer);

    let app = Router::new()
        .route("/", get(handlers::html_handler))
        .route("/chat", get(handlers::chat_html_handler))
        .route("/style.css", get(handlers::css_handler))
        .route("/client.js", get(handlers::js_handler))
        .route("/ws", any(handlers::ws_handler))
        .merge(auth_routes)
        .with_state(app_state);

    tracing::info!("Servidor corriendo en http://localhost:3000");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}