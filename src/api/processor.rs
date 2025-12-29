use std::pin::Pin;
use std::task::{Context, Poll};
use axum::body::{Body, BodyDataStream};
use bytes::{Bytes, BytesMut};
use futures_util::Stream;

const MAX_CHUNK_SIZE : usize = 64 * 1024 * 1024;

pub struct ChunkStream {
    inner: BodyDataStream,
    buffer: BytesMut,
}

impl ChunkStream {
    pub fn new(body: Body) -> Self {
        Self {
            inner: body.into_data_stream(),
            buffer: BytesMut::with_capacity(MAX_CHUNK_SIZE),
        }
    }
}



impl Stream for ChunkStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {

        loop {

            if self.buffer.len() >= MAX_CHUNK_SIZE {
                let chunk = self.buffer.split_to(MAX_CHUNK_SIZE).freeze();
                return Poll::Ready(Some(Ok(chunk)));
            }

            // poll BodyDataStream to pull bytes and buffer
            match Pin::new(&mut self.inner).poll_next(cx) {

                Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.extend_from_slice(&bytes);
                }

                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    ))));
                }

                Poll::Ready(None) => {
                    // End of stream return leftover bytes in the buffer.
                    if !self.buffer.is_empty() {
                        let chunk = self.buffer.split().freeze();
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                    return Poll::Ready(None);
                }

                Poll::Pending => return Poll::Pending,
            }
        }
    }
}