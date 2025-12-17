use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tokio::fs::{self, File};
use tokio_util::io::StreamReader;
use tracing::error;
use crate::models::DataNodeState;
use crate::helpers::get_chunk_path;
use futures_util::{TryStreamExt};

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


    let stream = body
        .into_data_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    let mut reader = StreamReader::new(stream);

    if let Err(e) = tokio::io::copy(&mut reader, &mut file).await {
        error!(%e, "stream write failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
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
