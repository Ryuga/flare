use axum::Router;
use axum::routing::{delete, get, head, put};
use crate::models::AppState;
use crate::storage;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/:bucket/*key", head(storage::head_object))
        .route("/:bucket/*key", get(storage::get_object))
        .route("/:bucket/*key", put(storage::put_object))
        .route("/:bucket/*key", delete(storage::delete_object))
        .with_state(state)
}