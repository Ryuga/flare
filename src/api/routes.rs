use axum::Router;
use axum::routing::{get, put};

use super::handlers::{get_object, put_object};

pub fn create_router() -> Router {
    Router::new()
        .route("/object/:key", put(put_object))
        .route("/object/:key", get(get_object))
}
