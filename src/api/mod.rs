mod routes;
mod handlers;
mod processor;
mod placement;
mod client;
mod metadata;
mod models;
mod streaming;

use std::{net::SocketAddr};
use tokio::net::TcpListener;
use tracing::info;
use crate::api::metadata::MetadataStore;
use crate::api::models::ApiState;

pub async fn start(){
    tracing_subscriber::fmt::init();

    let state = ApiState {
        metadata: MetadataStore::new(),
    };

    let app = routes::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}