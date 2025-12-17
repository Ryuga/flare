use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tokio::fs::{self, File};
use tracing::error;
use super::models::DataNodeState;
use super::helpers::{get_chunk_path, is_disk_full};
use futures_util::{TryStreamExt};
use hyper::Response;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

const MAX_CHUNK_SIZE: u64 = 64 * 1024 * 1024;


pub async fn put_chunk(
    State(state): State<DataNodeState>,
    Path(chunk_id): Path<String>,
    body: Body,
) -> impl IntoResponse {

    if chunk_id.len() < 8 {
        return (StatusCode::BAD_REQUEST, "invalid chunk id").into_response();
    }

    let path = get_chunk_path(&state.storage_dir, &chunk_id);

    if path.exists(){
        return StatusCode::CONFLICT.into_response();
    }

    let dir = path.parent().unwrap(); // replace later

    if let Err(e) = fs::create_dir_all(dir).await {
        error!(%e, "create chunk dir");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let tmp_path = path.with_extension("tmp");

    let mut file = match File::create(&tmp_path).await {
        Ok(f) => f,
        Err(e) => {
            error!(%e, "create temp file");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };


    let mut stream = body
        .into_data_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    let mut written: u64 = 0;

    while let Some(chunk) = stream.try_next().await.unwrap_or(None) {
        written += chunk.len() as u64;

        if written > MAX_CHUNK_SIZE {
            drop(file);
            let _ = fs::remove_file(&tmp_path).await;
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }

        if let Err(e) = file.write_all(&chunk).await {
            drop(file);
            let _ = fs::remove_file(&tmp_path).await;

            if is_disk_full(&e) {
                return StatusCode::INSUFFICIENT_STORAGE.into_response();
            }

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }


    // Ensure data is flushed
    if let Err(e) = file.sync_all().await {
        error!(%e, "fsync failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Atomic rename
    if let Err(e) = fs::rename(&tmp_path, &path).await {
        error!(%e, "rename failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::CREATED.into_response()
}

pub async fn get_chunk(
    State(state): State<DataNodeState>,
    Path(chunk_id): Path<String>,
) -> impl IntoResponse {
    let path = get_chunk_path(&state.storage_dir, &chunk_id);

    match File::open(&path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from_stream(stream))
                .unwrap()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}


pub async fn delete_chunk(
    State(state): State<DataNodeState>,
    Path(chunk_id): Path<String>,
) -> impl IntoResponse {
    let path = get_chunk_path(&state.storage_dir, &chunk_id);

    match fs::remove_file(&path).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

