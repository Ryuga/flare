use axum::Router;
use axum::routing::{delete, get, put};
use super::models::{DataNodeState};
use super::handlers::{delete_chunk, get_chunk, put_chunk};
pub fn create_router(state: DataNodeState) -> Router {
    Router::new()
        .route("/chunk/:id", put(put_chunk))
        .route("/chunk/:id", get(get_chunk))
        .route("/chunk/:id", delete(delete_chunk))

        .with_state(state)
}