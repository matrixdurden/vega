//! Turns a profile into a ready-to-run java command line.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Stdio,
};

use crate::{
    Error, Result,
    account::Account,
    fabric, java,
    mojang::{self, AssetIndex, VersionJson},
    net::{Download, Http, Progress, ProgressEvent, Stage},
    paths::Paths,
    profile::{Loader, Profile},
};

/// The flags Mojang's own launcher uses: G1 tuned for short pauses.
const JVM_FLAGS: &[&str] = &[
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+UseG1GC",
    "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20",
    "-XX:MaxGCPauseMillis=50",
    "-XX:G1HeapRegionSize=32M",
];

#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub java: PathBuf,
    pub args: Vec<String>,
    pub game_dir: PathBuf,
}

impl LaunchPlan {
    pub fn command(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.java);
        cmd.args(&self.args)
            .current_dir(&self.game_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        cmd
    }
}

pub struct LaunchRequest<'a> {
    pub profile: &'a Profile,
    pub account: &'a Account,
    pub memory_mb: u32,
}

/// Downloads whatever is missing (game, libraries, assets, Java) and builds
/// the command line. Everything already on disk is reused.
pub async fn prepare(
    http: &Http,
    paths: &Paths,
    request: &LaunchRequest<'_>,
    progress: &Progress,
) -> Result<LaunchPlan> {
    let LaunchRequest { profile, account, memory_mb } = *request;

    progress(ProgressEvent { stage: Stage::Metadata, done: 0, total: 1 });
    let version = resolve_version(http, paths, profile).await?;
    progress(ProgressEvent { stage: Stage::Metadata, done: 1, total: 1 });

    let libraries_dir = paths.libraries();
    let jar = paths.version_jar(&profile.game_version);
    let client = version
        .downloads
        .as_ref()
        .and_then(|downloads| downloads.client.as_ref())
        .ok_or_else(|| Error::other(format!("Minecraft {} has no client download", profile.game_version)))?;

    let mut downloads = vec![Download::new(&client.url, &jar).sha1(client.sha1.clone()).size(client.size)];
    let mut classpath = Vec::new();
    let mut native_jars = Vec::new();
    for library in version.libraries.iter().filter(|library| library.applies()) {
        if let Some(download) = library.artifact(&libraries_dir) {
            classpath.push(download.path.clone());
            downloads.push(download);
        }
        if let Some(download) = library.native(&libraries_dir) {
            native_jars.push(download.path.clone());
            downloads.push(download);
        }
    }
    let mut seen = HashSet::new();
    downloads.retain(|download| seen.insert(download.path.clone()));
    let mut seen = HashSet::new();
    classpath.retain(|path| seen.insert(path.clone()));
    classpath.push(jar);
    http.fetch_all(downloads, Stage::Libraries, progress).await?;

    let index_ref = version.asset_index.as_ref().ok_or_else(|| Error::other("version has no asset index"))?;
    let index_path = paths.assets().join("indexes").join(format!("{}.json", index_ref.id));
    let index: AssetIndex = http.json_cached(&index_ref.url, &index_path, Some(&index_ref.sha1)).await?;
    http.fetch_all(mojang::asset_downloads(&index, &paths.assets()), Stage::Assets, progress).await?;

    let game_dir = paths.instance(&profile.id);
    tokio::fs::create_dir_all(&game_dir).await?;
    let game_assets = mojang::materialize_legacy_assets(&index, &index_ref.id, paths, &game_dir).await?;

    let component = version.java_version.as_ref().map_or(java::LEGACY_COMPONENT, |java| java.component.as_str());
    let java = java::ensure(http, paths, component, progress).await?;

    progress(ProgressEvent { stage: Stage::Starting, done: 0, total: 1 });
    let natives_dir = paths.natives(&version.id);
    extract_natives(native_jars, natives_dir.clone()).await?;

    let separator = if cfg!(windows) { ";" } else { ":" };
    let classpath = classpath.iter().map(|path| path_str(path)).collect::<Vec<_>>().join(separator);
    let vars: HashMap<&str, String> = HashMap::from([
        ("auth_player_name", account.username.clone()),
        ("auth_uuid", account.uuid.clone()),
        ("auth_access_token", account.access_token.clone()),
        ("auth_session", account.access_token.clone()),
        ("auth_xuid", "0".into()),
        ("clientid", String::new()),
        ("user_type", account.user_type.into()),
        ("user_properties", "{}".into()),
        ("version_name", version.id.clone()),
        ("version_type", version.kind.clone().unwrap_or_else(|| "release".into())),
        ("game_directory", path_str(&game_dir)),
        ("assets_root", path_str(&paths.assets())),
        ("game_assets", path_str(&game_assets)),
        ("assets_index_name", index_ref.id.clone()),
        ("natives_directory", path_str(&natives_dir)),
        ("library_directory", path_str(&libraries_dir)),
        ("classpath_separator", separator.into()),
        ("classpath", classpath),
        ("launcher_name", "vega".into()),
        ("launcher_version", env!("CARGO_PKG_VERSION").into()),
    ]);

    let arguments = version.arguments.clone().unwrap_or_default();
    let mut jvm = mojang::active_args(&arguments.jvm);
    let mut game = mojang::active_args(&arguments.game);
    // Pre-1.13 versions describe their arguments as one string.
    if let Some(legacy) = &version.minecraft_arguments {
        jvm.extend(["-Djava.library.path=${natives_directory}", "-cp", "${classpath}"].map(String::from));
        game.extend(legacy.split_whitespace().map(String::from));
    }

    let main_class = version.main_class.clone().ok_or_else(|| Error::other("version has no main class"))?;
    let mut args = vec![format!("-Xmx{memory_mb}M"), format!("-Xms{}M", memory_mb.min(1024))];
    args.extend(JVM_FLAGS.iter().map(|flag| flag.to_string()));
    args.extend(jvm.iter().map(|arg| substitute(arg, &vars)));
    args.push(main_class);
    args.extend(game.iter().map(|arg| substitute(arg, &vars)));

    Ok(LaunchPlan { java, args, game_dir })
}

async fn resolve_version(http: &Http, paths: &Paths, profile: &Profile) -> Result<VersionJson> {
    let manifest = mojang::manifest(http, paths).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|version| version.id == profile.game_version)
        .ok_or_else(|| Error::other(format!("unknown Minecraft version {}", profile.game_version)))?;
    let vanilla: VersionJson = http.json_cached(&entry.url, &paths.version_json(&entry.id), Some(&entry.sha1)).await?;

    match profile.loader {
        Loader::Vanilla => Ok(vanilla),
        Loader::Fabric => {
            let loader = match &profile.loader_version {
                Some(version) => version.clone(),
                None => fabric::latest_loader(http, &profile.game_version).await?,
            };
            Ok(fabric::profile(http, paths, &profile.game_version, &loader).await?.merge_parent(vanilla))
        }
    }
}

/// Unpacks pre-1.19 native libraries (LWJGL 2 and friends).
async fn extract_natives(jars: Vec<PathBuf>, dir: PathBuf) -> Result<()> {
    if jars.is_empty() {
        return Ok(());
    }
    tokio::task::spawn_blocking(move || -> Result<()> {
        std::fs::create_dir_all(&dir)?;
        for jar in jars {
            let mut archive = zip::ZipArchive::new(std::fs::File::open(jar)?)?;
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let Some(name) = entry.enclosed_name() else { continue };
                if entry.is_dir() || name.starts_with("META-INF") {
                    continue;
                }
                let out = dir.join(name);
                if out.exists() {
                    continue;
                }
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::io::copy(&mut entry, &mut std::fs::File::create(out)?)?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|err| Error::other(err.to_string()))?
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Replaces `${name}` placeholders; unknown ones are left untouched.
fn substitute(arg: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            rest = &rest[start..];
            break;
        };
        match vars.get(&after[..end]) {
            Some(value) => out.push_str(value),
            None => out.push_str(&rest[start..start + 2 + end + 1]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_placeholders() {
        let vars = HashMap::from([("a", "1".to_string()), ("natives_directory", "/n".to_string())]);
        assert_eq!(substitute("-Djava.library.path=${natives_directory}", &vars), "-Djava.library.path=/n");
        assert_eq!(substitute("${a}-${a}", &vars), "1-1");
        assert_eq!(substitute("${missing} ${a}", &vars), "${missing} 1");
        assert_eq!(substitute("broken ${a", &vars), "broken ${a");
    }
}
