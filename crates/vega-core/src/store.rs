//! Small helpers for the JSON files Vega keeps on disk.

use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};

use crate::Result;

/// Writes through a temporary file so a crash never leaves half a file behind.
pub async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".part");
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub async fn read_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(err) => Err(err.into()),
    }
}

pub async fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    write_atomic(path, &serde_json::to_vec_pretty(value)?).await
}
