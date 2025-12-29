use axum::{
    body::Body,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::TryStreamExt;
use uuid::Uuid;
use crate::api::processor::ChunkStream;
use super::{
    client::DataNodeClient,
    placement::{select_node, DataNode},
};

pub async fn put_object(
    Path(key): Path<String>,
    body: Body,
) -> impl IntoResponse {
    // Hardcoded datanodes for now
    let nodes = vec![
        DataNode {
            base_url: "http://127.0.0.1:9000".into(),
        },
    ];

    let client = DataNodeClient::new();
    let mut stream = ChunkStream::new(body);

    // let mut chunk_index = 0;

    while let Some(chunk) = match stream.try_next().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("stream error: {e}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    } {
        let chunk_id = format!("{}-{}", key, Uuid::new_v4());
        let node = select_node(&nodes);

        if let Err(e) = client
            .put_chunk(&node.base_url, &chunk_id, chunk)
            .await
        {
            eprintln!("chunk upload failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        // chunk_index += 1;
    }

    StatusCode::CREATED.into_response()
}

pub async fn get_object(
    Path(_key): Path<String>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "GET not implemented")
}
