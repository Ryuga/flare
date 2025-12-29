use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct DataNode {
    pub base_url: String,
}

pub fn select_node(nodes: &[DataNode]) -> &DataNode {
    let idx = NEXT.fetch_add(1, Ordering::Relaxed) % nodes.len();
    &nodes[idx]
}
