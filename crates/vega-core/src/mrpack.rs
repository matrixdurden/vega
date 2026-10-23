//! Modrinth modpacks (`.mrpack`): https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack

use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    Error, Result,
    modrinth::InstalledMod,
    net::{Download, Http, Progress, Stage},
    store,
};

const API: &str = "https://api.modrinth.com/v2";

/// What installing a pack leaves behind for the profile.
pub struct PackInstall {
    pub loader_version: String,
    pub mods: Vec<InstalledMod>,
}

/// Sorts Minecraft versions like `26.3`, `1.21.11`, `1.21.8` newest first.
pub fn sort_newest_first(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_versions(b, a));
}

fn compare_versions(a: &str, b: &str) -> Ordering {
    let parts = |v: &str| v.split(['.', '-']).map(|p| p.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>();
    parts(a).cmp(&parts(b))
}

#[derive(Deserialize)]
struct PackVersion {
    version_type: String,
    game_versions: Vec<String>,
    files: Vec<PackFile>,
}

#[derive(Deserialize)]
struct PackFile {
    url: String,
    primary: bool,
    filename: String,
    hashes: Hashes,
}

#[derive(Deserialize)]
struct Hashes {
    sha1: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
    #[serde(default)]
    name: String,
    files: Vec<IndexFile>,
    dependencies: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndexFile {
    path: String,
    hashes: Hashes,
    #[serde(default)]
    env: Option<HashMap<String, String>>,
    downloads: Vec<String>,
    file_size: u64,
}

#[derive(Deserialize)]
struct Project {
    id: String,
    title: String,
    #[serde(default)]
    icon_url: Option<String>,
}

fn url(path: &str, params: &[(&str, &str)]) -> Result<String> {
    reqwest::Url::parse_with_params(&format!("{API}{path}"), params)
        .map(String::from)
        .map_err(|err| Error::other(err.to_string()))
}

async fn fabric_versions(http: &Http, slug: &str, game: Option<&str>) -> Result<Vec<PackVersion>> {
    let games = game.map(|game| format!(r#"["{game}"]"#));
    let mut params = vec![("loaders", r#"["fabric"]"#)];
    if let Some(games) = &games {
        params.push(("game_versions", games));
    }
    http.json(&url(&format!("/project/{slug}/version"), &params)?).await
}

/// Minecraft versions a Fabric modpack has builds for, newest first.
pub async fn game_versions(http: &Http, slug: &str) -> Result<Vec<String>> {
    let mut versions: Vec<String> =
        fabric_versions(http, slug, None).await?.into_iter().flat_map(|v| v.game_versions).collect();
    sort_newest_first(&mut versions);
    versions.dedup();
    Ok(versions)
}

/// Installs the newest stable build of a Fabric modpack for `game`.
pub async fn install(http: &Http, slug: &str, game: &str, instance: &Path, progress: &Progress) -> Result<PackInstall> {
    let versions = fabric_versions(http, slug, Some(game)).await?;
    let version = versions
        .iter()
        .find(|v| v.version_type == "release")
        .or(versions.first())
        .ok_or_else(|| Error::other(format!("{slug} has no Fabric build for Minecraft {game}")))?;
    let pack = version
        .files
        .iter()
        .find(|file| file.primary && file.filename.ends_with(".mrpack"))
        .or_else(|| version.files.iter().find(|file| file.filename.ends_with(".mrpack")))
        .ok_or_else(|| Error::other(format!("{slug} has no .mrpack file")))?;
    let bytes = http.bytes(&pack.url, Some(&pack.hashes.sha1)).await?;
    let (info, installed) = install_bytes(http, bytes, instance, progress).await?;
    if info.game_version != game {
        return Err(Error::other(format!("{slug} is not built for Minecraft {game}")));
    }
    Ok(installed)
}

/// What a pack file says about itself.
#[derive(Debug, Clone)]
pub struct PackInfo {
    pub name: String,
    pub game_version: String,
    pub loader_version: String,
}

/// Reads a pack's name, Minecraft version and Fabric version without
/// installing anything. Only Fabric packs are supported.
pub fn inspect(bytes: &[u8]) -> Result<PackInfo> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let index: Index = serde_json::from_reader(archive.by_name("modrinth.index.json")?)?;
    pack_info(&index)
}

fn pack_info(index: &Index) -> Result<PackInfo> {
    let game_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .ok_or_else(|| Error::other("the modpack does not say which Minecraft version it needs"))?;
    let loader_version = index
        .dependencies
        .get("fabric-loader")
        .cloned()
        .ok_or_else(|| Error::other("only Fabric modpacks are supported for now"))?;
    let name = if index.name.trim().is_empty() { "Modpack".to_owned() } else { index.name.trim().to_owned() };
    Ok(PackInfo { name, game_version, loader_version })
}

/// Installs a pack file (its downloads and overrides) into `instance`.
pub async fn install_bytes(
    http: &Http,
    bytes: Vec<u8>,
    instance: &Path,
    progress: &Progress,
) -> Result<(PackInfo, PackInstall)> {
    let (index, overrides) =
        tokio::task::spawn_blocking(move || read_pack(bytes)).await.map_err(|err| Error::other(err.to_string()))??;
    let info = pack_info(&index)?;
    let loader_version = info.loader_version.clone();

    let mut downloads = Vec::new();
    let mut mods = Vec::new();
    for file in &index.files {
        let client_side = file.env.as_ref().and_then(|env| env.get("client")).map(String::as_str);
        if client_side == Some("unsupported") {
            continue;
        }
        let path = safe_path(&file.path)?;
        let url = file.downloads.first().ok_or_else(|| Error::other(format!("{} has no download", file.path)))?;
        downloads.push(
            Download::new(url, instance.join(&path)).sha1(Some(file.hashes.sha1.clone())).size(Some(file.file_size)),
        );
        if path.starts_with("mods") {
            mods.push(pack_mod(&path, url));
        }
    }
    http.fetch_all(downloads, Stage::Mods, progress).await?;
    for (path, bytes) in overrides {
        store::write_atomic(&instance.join(path), &bytes).await?;
    }
    name_mods(http, &mut mods).await;
    Ok((info, PackInstall { loader_version, mods }))
}

/// Files to copy over the game directory, as (relative path, contents).
type Overrides = Vec<(PathBuf, Vec<u8>)>;

/// The index and the files to copy over the game directory: `overrides/`
/// first, then `client-overrides/` on top.
fn read_pack(bytes: Vec<u8>) -> Result<(Index, Overrides)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let index: Index = serde_json::from_reader(archive.by_name("modrinth.index.json")?)?;
    let mut common = Vec::new();
    let mut client = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name()?.into_owned();
        let (target, rest) = if let Some(rest) = name.strip_prefix("overrides/") {
            (&mut common, rest)
        } else if let Some(rest) = name.strip_prefix("client-overrides/") {
            (&mut client, rest)
        } else {
            continue;
        };
        let path = safe_path(rest)?;
        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data)?;
        target.push((path, data));
    }
    common.extend(client);
    Ok((index, common))
}

/// Rejects absolute paths and `..` so a pack cannot write outside its profile.
fn safe_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty() || !path.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(Error::other(format!("unsafe path in modpack: {}", path.display())));
    }
    Ok(path)
}

/// Modrinth CDN links look like `.../data/<project>/versions/<version>/<file>`.
fn pack_mod(path: &Path, url: &str) -> InstalledMod {
    let parts: Vec<&str> = url.split('/').collect();
    let after = |key: &str| parts.iter().position(|p| *p == key).and_then(|i| parts.get(i + 1)).map(|s| s.to_string());
    let filename = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    InstalledMod {
        project_id: after("data").unwrap_or_else(|| filename.clone()),
        version_id: after("versions").unwrap_or_default(),
        version_number: String::new(),
        title: filename.trim_end_matches(".jar").to_owned(),
        icon_url: None,
        filename,
        dependency: false,
        enabled: true,
    }
}

/// Fills in real names and icons with one request; filenames stay if it fails.
async fn name_mods(http: &Http, mods: &mut [InstalledMod]) {
    let ids: Vec<&str> = mods.iter().map(|m| m.project_id.as_str()).collect();
    let Ok(ids) = serde_json::to_string(&ids) else { return };
    let Ok(url) = url("/projects", &[("ids", &ids)]) else { return };
    let Ok(projects) = http.json::<Vec<Project>>(&url).await else { return };
    let projects: HashMap<_, _> = projects.into_iter().map(|p| (p.id.clone(), p)).collect();
    for m in mods.iter_mut() {
        if let Some(project) = projects.get(&m.project_id) {
            m.title = project.title.clone();
            m.icon_url = project.icon_url.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_versions() {
        let mut v: Vec<String> = ["1.21.8", "26.3", "1.21.11", "26.1.2", "1.21.4"].map(String::from).to_vec();
        sort_newest_first(&mut v);
        assert_eq!(v, ["26.3", "26.1.2", "1.21.11", "1.21.8", "1.21.4"]);
    }

    #[test]
    fn rejects_escaping_paths() {
        assert!(safe_path("mods/sodium.jar").is_ok());
        assert!(safe_path("../evil.jar").is_err());
        assert!(safe_path("/etc/passwd").is_err());
        assert!(safe_path("config/../../x").is_err());
    }

    #[test]
    fn reads_ids_from_cdn_links() {
        let m =
            pack_mod(Path::new("mods/sodium.jar"), "https://cdn.modrinth.com/data/AANobbMI/versions/abc123/sodium.jar");
        assert_eq!((m.project_id.as_str(), m.version_id.as_str(), m.title.as_str()), ("AANobbMI", "abc123", "sodium"));
    }
}
