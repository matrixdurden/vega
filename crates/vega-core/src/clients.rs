//! Ready-made clients: a profile with a client or modpack installed in one go.

use std::{cmp::Ordering, collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    Error, Result, fabric,
    modrinth::InstalledMod,
    mrpack,
    net::{Download, Http, Progress},
};

const METEOR_API: &str = "https://meteorclient.com/api";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientKind {
    /// Cheat/utility clients; most servers ban them.
    Utility,
    Performance,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ClientKind,
    pub homepage: &'static str,
    /// Minecraft versions it can be installed for, newest first.
    pub versions: Vec<String>,
}

struct Entry {
    id: &'static str,
    name: &'static str,
    kind: ClientKind,
    homepage: &'static str,
    source: Source,
}

enum Source {
    Meteor,
    /// A Fabric modpack on Modrinth, by slug.
    Modpack(&'static str),
}

const CATALOG: &[Entry] = &[
    Entry {
        id: "meteor",
        name: "Meteor Client",
        kind: ClientKind::Utility,
        homepage: "https://meteorclient.com",
        source: Source::Meteor,
    },
    Entry {
        id: "fabulously-optimized",
        name: "Fabulously Optimized",
        kind: ClientKind::Performance,
        homepage: "https://modrinth.com/modpack/fabulously-optimized",
        source: Source::Modpack("fabulously-optimized"),
    },
];

/// What installing a client leaves behind for the profile.
pub struct ClientInstall {
    pub loader_version: String,
    pub mods: Vec<InstalledMod>,
}

/// Every client with the versions it supports right now. A source that
/// cannot be reached shows up with no versions instead of failing the list.
pub async fn catalog(http: &Http) -> Vec<ClientInfo> {
    let versions = futures::future::join_all(CATALOG.iter().map(|entry| async move {
        match entry.source {
            Source::Meteor => meteor_versions(http).await,
            Source::Modpack(slug) => mrpack::game_versions(http, slug).await,
        }
        .unwrap_or_default()
    }))
    .await;
    CATALOG
        .iter()
        .zip(versions)
        .map(|(entry, versions)| ClientInfo {
            id: entry.id,
            name: entry.name,
            kind: entry.kind,
            homepage: entry.homepage,
            versions,
        })
        .collect()
}

pub fn name(id: &str) -> Option<&'static str> {
    CATALOG.iter().find(|entry| entry.id == id).map(|entry| entry.name)
}

/// Installs a client into a profile's game directory.
pub async fn install(http: &Http, id: &str, game: &str, instance: &Path, progress: &Progress) -> Result<ClientInstall> {
    let entry =
        CATALOG.iter().find(|entry| entry.id == id).ok_or_else(|| Error::other(format!("unknown client {id}")))?;
    match entry.source {
        Source::Meteor => install_meteor(http, game, instance).await,
        Source::Modpack(slug) => mrpack::install(http, slug, game, instance, progress).await,
    }
}

#[derive(Deserialize)]
struct MeteorStats {
    builds: HashMap<String, u32>,
}

async fn meteor_versions(http: &Http) -> Result<Vec<String>> {
    let stats: MeteorStats = http.json(&format!("{METEOR_API}/stats")).await?;
    let mut versions: Vec<String> = stats.builds.into_keys().collect();
    sort_newest_first(&mut versions);
    Ok(versions)
}

async fn install_meteor(http: &Http, game: &str, instance: &Path) -> Result<ClientInstall> {
    let loader_version = fabric::latest_loader(http, game).await?;
    let url = reqwest::Url::parse_with_params(&format!("{METEOR_API}/download"), &[("version", game)])
        .map_err(|err| Error::other(err.to_string()))?;
    let filename = format!("meteor-client-{game}.jar");
    http.fetch(&Download::new(url.as_str(), instance.join("mods").join(&filename))).await?;
    Ok(ClientInstall {
        loader_version,
        mods: vec![InstalledMod {
            project_id: "meteor-client".into(),
            version_id: game.into(),
            version_number: game.into(),
            title: "Meteor Client".into(),
            icon_url: None,
            filename,
            dependency: false,
            enabled: true,
        }],
    })
}

/// Sorts Minecraft versions like `26.3`, `1.21.11`, `1.21.8` newest first.
pub fn sort_newest_first(versions: &mut [String]) {
    versions.sort_by(|a, b| compare_versions(b, a));
}

fn compare_versions(a: &str, b: &str) -> Ordering {
    let parts = |v: &str| v.split(['.', '-']).map(|p| p.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>();
    parts(a).cmp(&parts(b))
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
}
