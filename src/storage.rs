use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use tokio::fs;
use tracing::{error, info};
use crate::models::{AppState, ObjMeta};

pub async fn get_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {

    let key_db = format!("{}/{}", bucket, key.trim_start_matches('/'));

    match state.db.get(key_db.as_bytes()) {
        Ok(Some(value)) => match serde_json::from_slice::<ObjMeta>(&value) {
            Ok(meta) => match fs::read(meta.path).await {
                Ok(bytes) => {
                    let mut res = (StatusCode::OK, bytes).into_response();
                    res.headers_mut()
                        .insert("etag", meta.etag.parse().unwrap());
                    res
                }
                Err(e) => {
                    error!(%e, "read file");
                    (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
                }
            },
            Err(e) => {
                error!(%e, "deserialize meta");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, "not found").into_response(),
        Err(e) => {
            error!(%e, "db get");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}


pub async fn put_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    body: Bytes,
) -> impl IntoResponse {
    // TODO: This reads the whole body into memory - replace with streaming for production.
    let key = key.trim_start_matches('/');
    let bucket_dir = state.storage_dir.join(&bucket);
    if let Err(e) = fs::create_dir_all(&bucket_dir).await {
        error!(%e, "create bucket dir");
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response();
    }

    // compute etag (sha256 hex)
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let etag = hex::encode(hasher.finalize());

    // safe filename
    let filename = format!("{}-{}", etag, sanitize_filename::sanitize(key));
    let path = bucket_dir.join(filename);

    if let Err(e) = fs::write(&path, &body).await {
        error!(%e, "write file");
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response();
    }

    // store metadata in db
    let meta = ObjMeta {
        path: path.to_string_lossy().into_owned(),
        size: body.len() as u64,
        etag: etag.clone(),
    };

    let key_db = format!("{}/{}", bucket, key);
    match serde_json::to_vec(&meta) {
        Ok(serialized) => {
            if let Err(e) = state.db.put(key_db.as_bytes(), &serialized) {
                error!(%e, "db put");
                return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response();
            }
        }
        Err(e) => {
            error!(%e, "serialize meta");
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response();
        }
    }

    let mut res = (StatusCode::OK, "OK").into_response();
    // return ETag header
    res.headers_mut()
        .insert("etag", etag.parse().unwrap());
    res
}

pub async fn head_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {
    let key_db = format!("{}/{}", bucket, key.trim_start_matches('/'));
    match state.db.get(key_db.as_bytes()) {
        Ok(Some(value)) => match serde_json::from_slice::<ObjMeta>(&value) {
            Ok(meta) => {
                let mut res = (StatusCode::OK, "").into_response();
                res.headers_mut()
                    .insert("etag", meta.etag.parse().unwrap());
                res.headers_mut().insert(
                    "content-length",
                    meta.size.to_string().parse().unwrap(),
                );
                res
            }
            Err(e) => {
                error!(%e, "deserialize meta");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, "not found").into_response(),
        Err(e) => {
            error!(%e, "db get");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

pub async fn delete_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {
    let key_db = format!("{}/{}", bucket, key.trim_start_matches('/'));
    info!("Received request to delete object");
    match state.db.get(key_db.as_bytes()) {
        Ok(Some(value)) => match serde_json::from_slice::<ObjMeta>(&value) {
            Ok(meta) => {
                // delete file
                if let Err(e) = fs::remove_file(&meta.path).await {
                    error!(%e, "removed file");
                }
                if let Err(e) = state.db.delete(key_db.as_bytes()) {
                    error!(%e, "db delete");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response();
                }
                (StatusCode::OK, "file deleted").into_response()
            }
            Err(e) => {
                error!(%e, "failed to deserialize object metadata");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete object").into_response()
            }
        },

        Ok(None) => (StatusCode::NOT_FOUND, "file not found").into_response(),

        Err(e) => {
            error!(%e, "db get");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}