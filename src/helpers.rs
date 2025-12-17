use std::path::PathBuf;


/// Constructs chunk path from chunk id.
/// format: store/ab/cd/<chunk_id>. ab, cd - first 2 chars of chunk id
/// We store chunks under subfolders as we don't want filesystem slowdown
/// Subfolder categorization would allow faster lookups and garbage collection.
pub fn get_chunk_path(base: &PathBuf, chunk_id: &str) -> PathBuf {
    let p1 = &chunk_id[0..2];
    let p2 = &chunk_id[2..4];
    base.join("chunks").join(p1).join(p2).join(chunk_id)
}
