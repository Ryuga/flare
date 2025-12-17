mod routes;
mod models;
mod helpers;
mod filesystem;
mod handlers;

use std::{net::SocketAddr};
use tokio::net::TcpListener;
use tracing::{info};
use self::filesystem::init_storage_dir;
use self::models::DataNodeState;

pub async fn start() {
    tracing_subscriber::fmt::init();

    let storage_dir = init_storage_dir("./data")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to initialize storage directory: {e}");
            std::process::exit(1);
        });

    println!("Using storage dir: {:?}", storage_dir);

    let state = DataNodeState {
        storage_dir,
    };


    let app = routes::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

