
 // Parallel streaming - Allows fetching chunks of an object in parallel
 // Fetch chunks in parallel from different data nodes, but stream the response
 // sequentially to preserve order and avoid buffering the full object.

use bytes::Bytes;
use futures_core::stream::Stream;
use reqwest::Client;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

use crate::api::metadata::ChunkMeta;
use futures_util::TryStreamExt;

const CHANNEL_BUFFER: usize = 8;

// Channel receiver to Stream so we can combine with other streams and using in pipeline
// While stream is being polled, it must stay in fixed memory address so we Pin.
 struct ChunkChannel {
    rx: mpsc::Receiver<Result<Bytes, std::io::Error>>,
}

impl Stream for ChunkChannel {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_recv(cx) // consuming message would mut rx
    }
}


struct OrderedStream {
    streams: Vec<ChunkChannel>,
    index: usize,
}

impl Stream for OrderedStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {


        while self.index < self.streams.len() {
            let idx = self.index;
            match Pin::new(&mut self.streams[idx]).poll_next(cx) {
                Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                Poll::Ready(None) => {
                    self.index += 1;
                    continue;
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(None)
    }
}


pub fn parallel_stream_object(chunks: Vec<ChunkMeta>)
    -> impl Stream<Item = Result<Bytes, std::io::Error>> {

    let client = Client::new();

    let mut channels = Vec::new();

    for chunk in chunks {
        // Bounded channel so we don't buffer too much, creates backpressure.
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);
        let client = client.clone(); // separate client for each channel (cheap, Arc-like)

        tokio::spawn(async move {
            let url = format!("{}/chunk/{}", chunk.node, chunk.chunk_id);

            let resp = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e,
                        )))
                        .await;
                    return;
                }
            };

            if !resp.status().is_success() {
                let _ = tx
                    .send(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "chunk fetch failed",
                    )))
                    .await;
                return;
            }

            let mut stream = resp
                .bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));


            while let Some(bytes) = match stream.try_next().await {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            } {
                if tx.send(Ok(bytes)).await.is_err() {
                    return;
                }
            }
        });

        channels.push(ChunkChannel { rx });
    }

    OrderedStream { streams: channels, index: 0 }
}
