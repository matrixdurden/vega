//! Small helpers for the JSON files Vega keeps on disk.

use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Serialize, de::DeserializeOwned};

use crate::Result;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes through a temporary file so a crash never leaves half a file
/// behind. Each write gets its own temporary name, so two games preparing
/// at once can download the same file without clobbering each other.
pub async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".{}.{}.part", std::process::id(), TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)));
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
