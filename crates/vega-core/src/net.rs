use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use futures::{StreamExt, TryStreamExt, stream};
use serde::{Serialize, de::DeserializeOwned};
use sha1::{Digest, Sha1};

use crate::{Error, Result, store};

const USER_AGENT: &str =
    concat!("matrixdurden/vega/", env!("CARGO_PKG_VERSION"), " (https://github.com/matrixdurden/vega)");
const PARALLEL_DOWNLOADS: usize = 32;
/// With exponential backoff this rides out about 7.5 seconds of network
/// trouble (a dropped Wi-Fi or a DNS hiccup) before giving up.
const ATTEMPTS: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Metadata,
    Libraries,
    Assets,
    Java,
    Mods,
    Starting,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub stage: Stage,
    pub done: u64,
    pub total: u64,
}

pub type Progress = Arc<dyn Fn(ProgressEvent) + Send + Sync>;

pub fn no_progress() -> Progress {
    Arc::new(|_| {})
}

#[derive(Debug, Clone)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub executable: bool,
}

impl Download {
    pub fn new(url: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self { url: url.into(), path: path.into(), sha1: None, size: None, executable: false }
    }

    pub fn sha1(mut self, sha1: Option<String>) -> Self {
        self.sha1 = sha1;
        self
    }

    pub fn size(mut self, size: Option<u64>) -> Self {
        self.size = size;
        self
    }

    pub fn executable(mut self, executable: bool) -> Self {
        self.executable = executable;
        self
    }
}

#[derive(Clone)]
pub struct Http {
    client: reqwest::Client,
}

impl Http {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(180))
            .build()?;
        Ok(Self { client })
    }

    /// The raw client, for requests that need more than a verified GET.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub async fn json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        Ok(serde_json::from_slice(&self.bytes(url, None).await?)?)
    }

    /// Reads `path` if it is already there (and matches `sha1`), otherwise
    /// downloads it first. Used for files that never change once published.
    pub async fn json_cached<T: DeserializeOwned>(&self, url: &str, path: &Path, sha1: Option<&str>) -> Result<T> {
        if let Ok(bytes) = tokio::fs::read(path).await
            && sha1.is_none_or(|expected| sha1_hex(&bytes).eq_ignore_ascii_case(expected))
        {
            return Ok(serde_json::from_slice(&bytes)?);
        }
        let bytes = self.bytes(url, sha1).await?;
        store::write_atomic(path, &bytes).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Fetches a fresh copy of a file that changes over time, falling back to
    /// the last saved copy when offline.
    pub async fn json_refreshed<T: DeserializeOwned>(&self, url: &str, path: &Path) -> Result<T> {
        match self.bytes(url, None).await {
            Ok(bytes) => {
                let parsed = serde_json::from_slice(&bytes)?;
                store::write_atomic(path, &bytes).await?;
                Ok(parsed)
            }
            Err(err) => match tokio::fs::read(path).await {
                Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
                Err(_) => Err(err),
            },
        }
    }

    /// GETs `url`, verifying `sha1` when given, retrying with backoff.
    pub async fn bytes(&self, url: &str, sha1: Option<&str>) -> Result<Vec<u8>> {
        let mut attempt = 1;
        loop {
            match self.bytes_once(url, sha1).await {
                Ok(bytes) => return Ok(bytes),
                Err(err) if attempt >= ATTEMPTS => return Err(err),
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(500 << (attempt - 1))).await;
                    attempt += 1;
                }
            }
        }
    }

    async fn bytes_once(&self, url: &str, sha1: Option<&str>) -> Result<Vec<u8>> {
        let bytes = self.client.get(url).send().await?.error_for_status()?.bytes().await?.to_vec();
        if let Some(expected) = sha1
            && !sha1_hex(&bytes).eq_ignore_ascii_case(expected)
        {
            return Err(Error::Checksum { url: url.to_owned() });
        }
        Ok(bytes)
    }

    /// Downloads one file unless an intact copy is already on disk.
    pub async fn fetch(&self, dl: &Download) -> Result<()> {
        if is_present(dl).await {
            return Ok(());
        }
        let bytes = self.bytes(&dl.url, dl.sha1.as_deref()).await?;
        store::write_atomic(&dl.path, &bytes).await?;
        if dl.executable {
            set_executable(&dl.path)?;
        }
        Ok(())
    }

    /// Downloads many files in parallel, reporting progress about a hundred
    /// times at most so the UI is not flooded by thousands of asset events.
    pub async fn fetch_all(&self, downloads: Vec<Download>, stage: Stage, progress: &Progress) -> Result<()> {
        // One blocking pass over the disk is much cheaper than thousands of
        // async metadata calls, and on a warm launch it is all that runs.
        let downloads = tokio::task::spawn_blocking(move || {
            downloads.into_iter().filter(|dl| !is_present_blocking(dl)).collect::<Vec<_>>()
        })
        .await
        .map_err(|err| Error::other(err.to_string()))?;
        let total = downloads.len() as u64;
        if total == 0 {
            return Ok(());
        }
        let step = (total / 100).max(1);
        let done = AtomicU64::new(0);
        progress(ProgressEvent { stage, done: 0, total });

        stream::iter(downloads)
            .map(|dl| {
                let done = &done;
                async move {
                    self.fetch(&dl).await?;
                    let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                    if n.is_multiple_of(step) || n == total {
                        progress(ProgressEvent { stage, done: n, total });
                    }
                    Ok::<_, Error>(())
                }
            })
            .buffer_unordered(PARALLEL_DOWNLOADS)
            .try_collect()
            .await
    }
}

/// Size is a cheap integrity check; full hashes are verified on download.
async fn is_present(dl: &Download) -> bool {
    match tokio::fs::metadata(&dl.path).await {
        Ok(meta) => meta.is_file() && dl.size.is_none_or(|size| meta.len() == size),
        Err(_) => false,
    }
}

fn is_present_blocking(dl: &Download) -> bool {
    match std::fs::metadata(&dl.path) {
        Ok(meta) => meta.is_file() && dl.size.is_none_or(|size| meta.len() == size),
        Err(_) => false,
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    hex::encode(Sha1::digest(bytes))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_: &Path) -> Result<()> {
    Ok(())
}
