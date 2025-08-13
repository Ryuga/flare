use std::path::PathBuf;
use std::sync::Arc;
use rocksdb::DB;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DB>,
    pub storage_dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct ObjMeta {
    pub path: String,
    pub size: u64,
    pub etag: String,
}