use std::path::{Path, PathBuf};
use tokio::fs;
use std::fs as std_fs;

pub async fn init_storage_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_buf = PathBuf::from(path.as_ref());

    // Ensure absolute path
    let abs_path = std_fs::canonicalize(&path_buf)
        .unwrap_or_else(|_| {
            panic!("failed to canonicalize storage dir: {:?}", path_buf);
        });

    // Create dir if needed
    fs::create_dir_all(&abs_path)
        .await
        .unwrap_or_else(|_| panic!("failed to create storage dir: {:?}", abs_path));

    abs_path
}