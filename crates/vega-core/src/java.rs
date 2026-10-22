//! Java runtimes from Mojang's own runtime manifest, the same builds the
//! official launcher uses. Nothing has to be installed on the system.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    Error, Result,
    net::{Download, Http, Progress, Stage},
    paths::Paths,
};

const RUNTIMES_URL: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
const MARKER: &str = ".vega-runtime";

/// Used by versions older than 1.17, whose JSON has no `javaVersion`.
pub const LEGACY_COMPONENT: &str = "jre-legacy";

type RuntimeIndex = HashMap<String, HashMap<String, Vec<RuntimeEntry>>>;

#[derive(Deserialize)]
struct RuntimeEntry {
    manifest: FileRef,
}

#[derive(Deserialize)]
struct FileRef {
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Deserialize)]
struct RuntimeManifest {
    files: HashMap<String, RuntimeFile>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RuntimeFile {
    File {
        #[serde(default)]
        executable: bool,
        downloads: FileDownloads,
    },
    Directory,
    Link {
        #[cfg_attr(not(unix), allow(dead_code))]
        target: String,
    },
}

#[derive(Deserialize)]
struct FileDownloads {
    raw: FileRef,
}

fn platform() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("linux"),
        ("linux", "x86") => Some("linux-i386"),
        ("windows", "x86_64") => Some("windows-x64"),
        ("windows", "x86") => Some("windows-x86"),
        ("windows", "aarch64") => Some("windows-arm64"),
        ("macos", "x86_64") => Some("mac-os"),
        ("macos", "aarch64") => Some("mac-os-arm64"),
        _ => None,
    }
}

fn java_binary(dir: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        dir.join("jre.bundle/Contents/Home/bin/java")
    } else if cfg!(windows) {
        // javaw has no console window
        dir.join("bin").join("javaw.exe")
    } else {
        dir.join("bin").join("java")
    }
}

/// Makes sure `component` (for example `java-runtime-delta`) is installed
/// and returns its java binary. Works offline once installed.
pub async fn ensure(http: &Http, paths: &Paths, component: &str, progress: &Progress) -> Result<PathBuf> {
    let dir = paths.runtimes().join(component);
    let marker = dir.join(MARKER);
    let installed = tokio::fs::read_to_string(&marker).await.ok();

    let platform = platform().ok_or_else(|| Error::other("Mojang does not publish Java for this platform yet"))?;
    let index: RuntimeIndex = match http.json(RUNTIMES_URL).await {
        Ok(index) => index,
        Err(_) if installed.is_some() => return Ok(java_binary(&dir)),
        Err(err) => return Err(err),
    };
    let entry = index
        .get(platform)
        .and_then(|runtimes| runtimes.get(component))
        .and_then(|builds| builds.first())
        .ok_or_else(|| Error::other(format!("no Java build for {component} on {platform}")))?;
    if installed.as_deref() == Some(entry.manifest.sha1.as_str()) {
        return Ok(java_binary(&dir));
    }

    let manifest: RuntimeManifest =
        http.json_cached(&entry.manifest.url, &dir.join("manifest.json"), Some(&entry.manifest.sha1)).await?;
    let mut downloads = Vec::new();
    let mut links = Vec::new();
    for (path, file) in manifest.files {
        match file {
            RuntimeFile::File { executable, downloads: files } => downloads.push(
                Download::new(files.raw.url, dir.join(&path))
                    .sha1(Some(files.raw.sha1))
                    .size(Some(files.raw.size))
                    .executable(executable),
            ),
            RuntimeFile::Directory => tokio::fs::create_dir_all(dir.join(&path)).await?,
            RuntimeFile::Link { target } => links.push((path, target)),
        }
    }
    http.fetch_all(downloads, Stage::Java, progress).await?;
    create_links(&dir, links)?;

    tokio::fs::write(&marker, &entry.manifest.sha1).await?;
    Ok(java_binary(&dir))
}

#[cfg(unix)]
fn create_links(dir: &Path, links: Vec<(String, String)>) -> Result<()> {
    for (path, target) in links {
        let link = dir.join(path);
        let _ = std::fs::remove_file(&link);
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::os::unix::fs::symlink(target, link)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_links(_: &Path, _: Vec<(String, String)>) -> Result<()> {
    Ok(())
}
