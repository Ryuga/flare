use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::http::Response;
use futures_util::TryStreamExt;
use reqwest::Client;
use uuid::Uuid;
use crate::api::metadata::{ChunkMeta, ObjectMeta};
use crate::api::models::ApiState;
use crate::api::processor::ChunkStream;
use crate::api::streaming::stream_object;
use super::{
    client::DataNodeClient,
    placement::{select_node, DataNode},
};

pub async fn put_object(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    body: Body,
) -> impl IntoResponse {
    // Hardcoded datanodes for now
    let nodes = vec![
        DataNode {
            base_url: "http://127.0.0.1:9000".into(),
        },
        DataNode{
            base_url: "http://127.0.0.1:9001".into(),
        }
    ];

    let client = DataNodeClient::new();
    let mut stream = ChunkStream::new(body);

    let mut chunks = Vec::new();
    let mut total_size: usize = 0;
    let mut chunk_index: usize = 0;


    while let Some(chunk) = match stream.try_next().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("stream error: {e}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    } {
        let chunk_id = format!("{}-{}", key, Uuid::new_v4());
        let node = select_node(&nodes);
        let size = chunk.len();


        if let Err(e) = client
            .put_chunk(&node.base_url, &chunk_id, chunk)
            .await
        {
            eprintln!("chunk upload failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        chunks.push(
            ChunkMeta{
                index: chunk_index,
                node: node.base_url.clone(),
                chunk_id,
                size,
            }
        );
        total_size += size;
        chunk_index += 1;
    }

    state.metadata.set(
        key,
        ObjectMeta{
            size: total_size,
            chunks,
        }
    );

    StatusCode::CREATED.into_response()
}

pub async fn get_object(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> impl IntoResponse {

    let meta = match state.metadata.get(&key) {
        Some(m) => m,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut chunks = meta.chunks.clone();
    chunks.sort_by_key(|c| c.index);

    let stream = stream_object(chunks);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Length", meta.size)
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn delete_object(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> impl IntoResponse {

    let meta = match state.metadata.get(&key) {
        Some(m) => m,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let client = Client::new();

    for chunk in meta.chunks {
        let url = format!("{}/chunk/{}", chunk.node, chunk.chunk_id);

        let resp = match client.delete(url).send().await {
            Ok(r) => r,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };

        if !resp.status().is_success() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    state.metadata.remove(&key);

    StatusCode::OK.into_response()
}
