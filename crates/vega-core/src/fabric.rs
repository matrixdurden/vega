//! Fabric loader profiles from meta.fabricmc.net.

use serde::Deserialize;

use crate::{Error, Result, mojang::VersionJson, net::Http, paths::Paths};

const META: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Deserialize)]
pub struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Deserialize)]
struct LoaderEntry {
    loader: LoaderInfo,
}

#[derive(Deserialize)]
struct LoaderInfo {
    version: String,
    stable: bool,
}

/// Minecraft versions Fabric supports.
pub async fn game_versions(http: &Http) -> Result<Vec<FabricGameVersion>> {
    http.json(&format!("{META}/versions/game")).await
}

/// The newest stable loader for `game`.
pub async fn latest_loader(http: &Http, game: &str) -> Result<String> {
    let entries: Vec<LoaderEntry> = http.json(&format!("{META}/versions/loader/{game}")).await?;
    entries
        .iter()
        .find(|entry| entry.loader.stable)
        .or(entries.first())
        .map(|entry| entry.loader.version.clone())
        .ok_or_else(|| Error::other(format!("Fabric does not support Minecraft {game}")))
}

/// The Fabric version JSON for a game and loader pair. It never changes once
/// published, so it is downloaded once and read from disk afterwards.
pub async fn profile(http: &Http, paths: &Paths, game: &str, loader: &str) -> Result<VersionJson> {
    let id = format!("fabric-loader-{loader}-{game}");
    let url = format!("{META}/versions/loader/{game}/{loader}/profile/json");
    http.json_cached(&url, &paths.version_json(&id), None).await
}
