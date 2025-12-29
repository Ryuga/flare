mod routes;
mod handlers;
mod processor;
mod placement;
mod client;

use std::{net::SocketAddr};
use tokio::net::TcpListener;
use tracing::info;

pub async fn start(){
    tracing_subscriber::fmt::init();

    let app = routes::create_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}