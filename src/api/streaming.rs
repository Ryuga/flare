use async_stream::try_stream;
use bytes::Bytes;
use futures_core::stream::Stream;
use futures_util::TryStreamExt;
use reqwest::Client;

use crate::api::metadata::ChunkMeta;

pub fn stream_object(
    chunks: Vec<ChunkMeta>,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
    let client = Client::new();

    try_stream! {
        for chunk in chunks {
            let url = format!("{}/chunk/{}", chunk.node, chunk.chunk_id);

            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            if !resp.status().is_success() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "fetch failed",
                ))?;
            }

            let mut stream = resp
                .bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

            while let Some(bytes) = stream.try_next().await? {
                yield bytes;
            }
        }
    }
}
