use tokio::sync::broadcast;
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState{
    pub tx : broadcast::Sender<String>,
    pub db : Pool<Sqlite>,
}