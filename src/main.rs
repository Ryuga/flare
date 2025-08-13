mod routes;
mod storage;
mod models;
mod helpers;
mod db;
mod filesystem;

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{info};
use crate::db::init_db;
use crate::filesystem::init_storage_dir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let storage_dir = init_storage_dir("./data").await;

    let db = init_db("./metadata.db");

    let state = models::AppState {
        db: Arc::new(db),
        storage_dir,
    };

    let app = routes::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

