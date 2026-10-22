//! Mojang's version manifest, version JSONs, libraries and assets.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    Result,
    net::{Download, Http},
    paths::Paths,
    rules::{self, Rule},
};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";
const LIBRARIES_URL: &str = "https://libraries.minecraft.net/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
    pub release_time: String,
}

pub async fn manifest(http: &Http, paths: &Paths) -> Result<VersionManifest> {
    http.json_refreshed(MANIFEST_URL, &paths.versions().join("version_manifest_v2.json")).await
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    #[serde(default)]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default)]
    pub downloads: Option<VersionDownloads>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Ruled { rules: Vec<Rule>, value: ArgValue },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    One(String),
    Many(Vec<String>),
}

/// Flattens arguments whose rules apply on this machine.
pub fn active_args(args: &[Argument]) -> Vec<String> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            Argument::Plain(value) => out.push(value.clone()),
            Argument::Ruled { rules, value } if rules::allowed(rules) => match value {
                ArgValue::One(value) => out.push(value.clone()),
                ArgValue::Many(values) => out.extend(values.iter().cloned()),
            },
            Argument::Ruled { .. } => {}
        }
    }
    out
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDownloads {
    #[serde(default)]
    pub client: Option<Artifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    /// Maven repository base, used by Fabric libraries that have no `downloads`.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Pre-1.19 native classifiers, keyed by OS name.
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

impl Library {
    pub fn applies(&self) -> bool {
        rules::allowed(&self.rules)
    }

    /// `group:artifact[:classifier]`, so a loader can replace a vanilla
    /// library with its own version.
    pub fn key(&self) -> String {
        let parts: Vec<&str> = self.name.split(':').collect();
        match parts.as_slice() {
            [group, artifact, _] => format!("{group}:{artifact}"),
            [group, artifact, _, classifier, ..] => format!("{group}:{artifact}:{classifier}"),
            _ => self.name.clone(),
        }
    }

    /// The jar that goes on the classpath.
    pub fn artifact(&self, libraries: &Path) -> Option<Download> {
        match &self.downloads {
            Some(downloads) => {
                let artifact = downloads.artifact.as_ref().filter(|a| !a.url.is_empty())?;
                let path = artifact.path.clone().or_else(|| maven_path(&self.name))?;
                Some(Download::new(&artifact.url, libraries.join(path)).sha1(artifact.sha1.clone()).size(artifact.size))
            }
            None => {
                let path = maven_path(&self.name)?;
                let base = self.url.as_deref().unwrap_or(LIBRARIES_URL);
                let url = format!("{}/{path}", base.trim_end_matches('/'));
                Some(Download::new(url, libraries.join(path)).sha1(self.sha1.clone()).size(self.size))
            }
        }
    }

    /// The legacy natives jar to extract before launch, if any.
    pub fn native(&self, libraries: &Path) -> Option<Download> {
        let arch = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
        let classifier = self.natives.as_ref()?.get(rules::os_name())?.replace("${arch}", arch);
        let artifact = self.downloads.as_ref()?.classifiers.as_ref()?.get(&classifier)?;
        let path = artifact.path.clone().or_else(|| maven_path(&format!("{}:{classifier}", self.name)))?;
        Some(Download::new(&artifact.url, libraries.join(path)).sha1(artifact.sha1.clone()).size(artifact.size))
    }
}

/// `group:artifact:version[:classifier][@ext]` to a repository path.
pub fn maven_path(name: &str) -> Option<String> {
    let (coords, ext) = name.split_once('@').unwrap_or((name, "jar"));
    let parts: Vec<&str> = coords.split(':').collect();
    let (group, artifact, version) = (parts.first()?, parts.get(1)?, parts.get(2)?);
    let file = match parts.get(3) {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.{ext}"),
        None => format!("{artifact}-{version}.{ext}"),
    };
    Some(format!("{}/{artifact}/{version}/{file}", group.replace('.', "/")))
}

impl VersionJson {
    /// Folds the parent (vanilla) version into a child (mod loader) version.
    /// The child's libraries win when both ship the same one.
    pub fn merge_parent(self, parent: VersionJson) -> VersionJson {
        let mut seen: HashSet<String> = self.libraries.iter().map(Library::key).collect();
        let mut libraries = self.libraries;
        libraries.extend(parent.libraries.into_iter().filter(|lib| seen.insert(lib.key())));

        let arguments = match (parent.arguments, self.arguments) {
            (Some(mut base), Some(child)) => {
                base.game.extend(child.game);
                base.jvm.extend(child.jvm);
                Some(base)
            }
            (base, child) => base.or(child),
        };

        VersionJson {
            id: self.id,
            inherits_from: None,
            main_class: self.main_class.or(parent.main_class),
            arguments,
            minecraft_arguments: self.minecraft_arguments.or(parent.minecraft_arguments),
            asset_index: self.asset_index.or(parent.asset_index),
            downloads: self.downloads.or(parent.downloads),
            libraries,
            java_version: self.java_version.or(parent.java_version),
            kind: self.kind.or(parent.kind),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub fn asset_object_path(assets: &Path, hash: &str) -> PathBuf {
    assets.join("objects").join(&hash[..2]).join(hash)
}

pub fn asset_downloads(index: &AssetIndex, assets: &Path) -> Vec<Download> {
    let mut seen = HashSet::new();
    index
        .objects
        .values()
        .filter(|object| seen.insert(object.hash.as_str()))
        .map(|object| {
            let url = format!("{RESOURCES_URL}/{}/{}", &object.hash[..2], object.hash);
            Download::new(url, asset_object_path(assets, &object.hash))
                .sha1(Some(object.hash.clone()))
                .size(Some(object.size))
        })
        .collect()
}

/// Versions before 1.7 read assets by name instead of by hash, so copy them
/// into the layout they expect. Returns the directory for `${game_assets}`.
pub async fn materialize_legacy_assets(
    index: &AssetIndex,
    index_id: &str,
    paths: &Paths,
    game_dir: &Path,
) -> Result<PathBuf> {
    let target = if index.map_to_resources {
        game_dir.join("resources")
    } else if index.is_virtual {
        paths.assets().join("virtual").join(index_id)
    } else {
        return Ok(paths.assets());
    };
    for (name, object) in &index.objects {
        let dest = target.join(name);
        if tokio::fs::try_exists(&dest).await? {
            continue;
        }
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(asset_object_path(&paths.assets(), &object.hash), &dest).await?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maven_paths() {
        assert_eq!(
            maven_path("net.fabricmc:fabric-loader:0.16.10").unwrap(),
            "net/fabricmc/fabric-loader/0.16.10/fabric-loader-0.16.10.jar"
        );
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3:natives-linux").unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-linux.jar"
        );
        assert_eq!(maven_path("a.b:c:1@zip").unwrap(), "a/b/c/1/c-1.zip");
    }

    #[test]
    fn child_libraries_override_parent() {
        let lib = |name: &str| Library {
            name: name.into(),
            downloads: None,
            url: None,
            sha1: None,
            size: None,
            rules: vec![],
            natives: None,
        };
        let version = |id: &str, libs: Vec<Library>| VersionJson {
            id: id.into(),
            inherits_from: None,
            main_class: Some(format!("{id}.Main")),
            arguments: None,
            minecraft_arguments: None,
            asset_index: None,
            downloads: None,
            libraries: libs,
            java_version: None,
            kind: None,
        };
        let parent = version("vanilla", vec![lib("org.ow2.asm:asm:9.0"), lib("com.mojang:brigadier:1.0")]);
        let child = version("fabric", vec![lib("org.ow2.asm:asm:9.7")]);
        let merged = child.merge_parent(parent);
        let names: Vec<_> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["org.ow2.asm:asm:9.7", "com.mojang:brigadier:1.0"]);
        assert_eq!(merged.main_class.as_deref(), Some("fabric.Main"));
    }
}
