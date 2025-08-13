use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use tracing::error;

pub fn handle_db_get<T: DeserializeOwned>(key: &[u8], value: Option<Vec<u8>>) -> Result<Option<T>, Response> {
    match value {
        Some(v) => match serde_json::from_slice::<T>(&v) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(e) => {
                error!(%e, "failed to deserialize meta");
                Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to parse value").into_response())
            }
        },
        None => Ok(None),
    }
}