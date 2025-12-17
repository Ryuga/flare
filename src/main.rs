mod routes;
mod models;
mod helpers;
mod filesystem;
mod datanode;

use std::{net::SocketAddr};
use tokio::net::TcpListener;
use tracing::{info};
use crate::filesystem::init_storage_dir;
use crate::models::DataNodeState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let storage_dir = init_storage_dir("./data").await;

    let state = DataNodeState {
        storage_dir,
    };


    let app = routes::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

