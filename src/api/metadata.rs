use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ChunkMeta {
    pub index: usize,
    pub size: usize,
    pub chunk_id: String,
    pub node: String,
}

#[derive(Clone)]
pub struct ObjectMeta {
    pub size: usize,
    pub chunks: Vec<ChunkMeta>
}

#[derive(Clone)]
pub struct MetadataStore {
    store: Arc<Mutex<HashMap<String, ObjectMeta>>>,
}

impl MetadataStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<ObjectMeta> {
        let guard = self.store.lock().unwrap();
        guard.get(key).cloned()
    }

    pub fn set(&self, key: String, meta: ObjectMeta) {
        let mut guard = self.store.lock().unwrap();
        guard.insert(key, meta);
    }

    pub fn remove(&self, key: &str) {
        let mut guard = self.store.lock().unwrap();
        guard.remove(key);
    }
}
