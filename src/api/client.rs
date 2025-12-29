use bytes::Bytes;
use reqwest::Client;
use std::{io, time::Duration};

pub struct DataNodeClient {
    client: Client,
}

impl DataNodeClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn put_chunk(
        &self,
        node_url: &str,
        chunk_id: &str,
        data: Bytes,
    ) -> Result<(), io::Error> {
        let url = format!("{}/chunk/{}", node_url, chunk_id);

        let res = self
            .client
            .put(url)
            .body(data)
            .send()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if !res.status().is_success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("datanode returned {}", res.status()),
            ));
        }

        Ok(())
    }
}
