use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn init_storage_dir<P: AsRef<Path>>(
    storage_path: P,
) -> Result<PathBuf, std::io::Error> {
    let path = storage_path.as_ref();

    // Create directory if missing
    fs::create_dir_all(path).await?;

    // Convert to absolute path WITHOUT resolving symlinks
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(abs_path)
}
