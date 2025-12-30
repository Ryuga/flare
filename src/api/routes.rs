use axum::Router;
use axum::routing::{get, put, delete};
use crate::api::models::ApiState;
use super::handlers::{get_object, put_object, delete_object};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/object/:key", put(put_object))
        .route("/object/:key", get(get_object))
        .route("/object/:key", delete(delete_object))
        .with_state(state)
}
