use axum::Router;
use axum::routing::{put};
use crate::models::{DataNodeState};
use crate::datanode::put_chunk;
pub fn create_router(state: DataNodeState) -> Router {
    Router::new()
        .route("/chunk/:id", put(put_chunk))
        .with_state(state)
}