//! Modrinth search and installs, including required dependencies.

use std::{collections::VecDeque, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    Error, Result,
    net::{Download, Http},
};

const API: &str = "https://api.modrinth.com/v2";
const LOADER: &str = "fabric";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SearchHit {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub author: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct Version {
    id: String,
    project_id: String,
    version_number: String,
    files: Vec<VersionFile>,
    #[serde(default)]
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct VersionFile {
    url: String,
    filename: String,
    primary: bool,
    size: u64,
    hashes: Hashes,
}

#[derive(Deserialize)]
struct Hashes {
    sha1: String,
}

#[derive(Deserialize)]
struct Dependency {
    version_id: Option<String>,
    project_id: Option<String>,
    dependency_type: String,
}

#[derive(Deserialize)]
struct Project {
    title: String,
    #[serde(default)]
    icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub project_id: String,
    pub version_id: String,
    pub version_number: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub filename: String,
    /// Pulled in because another mod needs it, not picked by the player.
    pub dependency: bool,
    /// Disabled mods stay on disk as `<filename>.disabled`.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

fn enabled_by_default() -> bool {
    true
}

fn url(path: &str, params: &[(&str, &str)]) -> Result<String> {
    reqwest::Url::parse_with_params(&format!("{API}{path}"), params)
        .map(String::from)
        .map_err(|err| Error::other(err.to_string()))
}

/// Fabric mods for `game`; an empty query lists the most downloaded ones.
pub async fn search(http: &Http, query: &str, game: &str) -> Result<Vec<SearchHit>> {
    let facets =
        serde_json::json!([["project_type:mod"], [format!("categories:{LOADER}")], [format!("versions:{game}")]]);
    search_with(http, query, facets).await
}

/// Fabric modpacks for any Minecraft version.
pub async fn search_modpacks(http: &Http, query: &str) -> Result<Vec<SearchHit>> {
    search_with(http, query, serde_json::json!([["project_type:modpack"], [format!("categories:{LOADER}")]])).await
}

async fn search_with(http: &Http, query: &str, facets: serde_json::Value) -> Result<Vec<SearchHit>> {
    let index = if query.trim().is_empty() { "downloads" } else { "relevance" };
    let url =
        url("/search", &[("query", query.trim()), ("facets", &facets.to_string()), ("index", index), ("limit", "30")])?;
    Ok(http.json::<SearchResponse>(&url).await?.hits)
}

enum Wanted {
    Project(String),
    Version(String),
}

/// Installs the newest compatible build of `project_id` and every required
/// dependency that is not installed yet. Returns what was added.
pub async fn install(
    http: &Http,
    mods_dir: &Path,
    game: &str,
    project_id: &str,
    installed: &mut Vec<InstalledMod>,
) -> Result<Vec<InstalledMod>> {
    let is_installed = |installed: &[InstalledMod], id: &str| installed.iter().any(|m| m.project_id == id);
    let mut queue = VecDeque::from([(Wanted::Project(project_id.to_owned()), false)]);
    let mut added = Vec::new();

    while let Some((wanted, dependency)) = queue.pop_front() {
        let version = match wanted {
            Wanted::Project(id) if is_installed(installed, &id) => continue,
            Wanted::Project(id) => newest_compatible(http, &id, game).await?,
            Wanted::Version(id) => http.json::<Version>(&format!("{API}/version/{id}")).await?,
        };
        if is_installed(installed, &version.project_id) {
            continue;
        }

        let file = version
            .files
            .iter()
            .find(|file| file.primary)
            .or(version.files.first())
            .ok_or_else(|| Error::other(format!("{} has no downloadable file", version.project_id)))?;
        let download = Download::new(&file.url, mods_dir.join(&file.filename))
            .sha1(Some(file.hashes.sha1.clone()))
            .size(Some(file.size));
        http.fetch(&download).await?;

        let project: Project = http.json(&format!("{API}/project/{}", version.project_id)).await?;
        let entry = InstalledMod {
            project_id: version.project_id.clone(),
            version_id: version.id.clone(),
            version_number: version.version_number.clone(),
            title: project.title,
            icon_url: project.icon_url,
            filename: file.filename.clone(),
            dependency,
            enabled: true,
        };
        installed.push(entry.clone());
        added.push(entry);

        for dep in version.dependencies.iter().filter(|dep| dep.dependency_type == "required") {
            match (&dep.project_id, &dep.version_id) {
                (Some(project), _) => queue.push_back((Wanted::Project(project.clone()), true)),
                (None, Some(version)) => queue.push_back((Wanted::Version(version.clone()), true)),
                (None, None) => {}
            }
        }
    }
    Ok(added)
}

async fn newest_compatible(http: &Http, project_id: &str, game: &str) -> Result<Version> {
    let loaders = format!(r#"["{LOADER}"]"#);
    let games = format!(r#"["{game}"]"#);
    let url = url(&format!("/project/{project_id}/version"), &[("loaders", &loaders), ("game_versions", &games)])?;
    let versions: Vec<Version> = http.json(&url).await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| Error::other(format!("{project_id} has no Fabric build for Minecraft {game}")))
}
