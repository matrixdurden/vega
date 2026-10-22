use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    Error, Result,
    account::{self, Account},
    fabric,
    launch::{self, LaunchPlan, LaunchRequest},
    modrinth::{self, InstalledMod, SearchHit},
    mojang,
    net::{Http, Progress},
    paths::Paths,
    profile::{Loader, Profile, ProfileDraft, Settings, World},
    store,
};

/// Profile sub-folders the UI may open.
const OPENABLE_FOLDERS: &[&str] = &["mods", "saves", "resourcepacks", "shaderpacks", "screenshots"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub id: String,
    pub release_time: String,
    pub fabric: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModResult {
    #[serde(flatten)]
    pub hit: SearchHit,
    pub installed: bool,
}

/// The one object a frontend talks to.
pub struct Launcher {
    http: Http,
    paths: Paths,
    /// Serialises writes to profiles.json and the mod indexes.
    write_lock: Mutex<()>,
}

impl Launcher {
    pub fn new(paths: Paths) -> Result<Self> {
        Ok(Self { http: Http::new()?, paths, write_lock: Mutex::new(()) })
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    /// The profile's game directory, or one of its known sub-folders.
    pub fn instance_dir(&self, profile_id: &str, folder: Option<&str>) -> Result<PathBuf> {
        let dir = self.paths.instance(profile_id);
        match folder {
            None => Ok(dir),
            Some(folder) if OPENABLE_FOLDERS.contains(&folder) => Ok(dir.join(folder)),
            Some(folder) => Err(Error::other(format!("unknown profile folder {folder}"))),
        }
    }

    pub async fn settings(&self) -> Result<Settings> {
        store::read_json(&self.paths.settings_file()).await
    }

    pub async fn save_settings(&self, mut settings: Settings) -> Result<Settings> {
        settings.username = settings.username.trim().to_owned();
        if !account::valid_username(&settings.username) {
            return Err(Error::other("username must be 3-16 letters, digits or underscores"));
        }
        settings.memory_mb = settings.memory_mb.clamp(1024, 32768);
        store::write_json(&self.paths.settings_file(), &settings).await?;
        Ok(settings)
    }

    pub async fn profiles(&self) -> Result<Vec<Profile>> {
        store::read_json(&self.paths.profiles_file()).await
    }

    async fn profile(&self, id: &str) -> Result<Profile> {
        self.profiles()
            .await?
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| Error::other(format!("profile {id} not found")))
    }

    async fn loader_version(&self, loader: Loader, game_version: &str) -> Result<Option<String>> {
        match loader {
            Loader::Fabric => Ok(Some(fabric::latest_loader(&self.http, game_version).await?)),
            Loader::Vanilla => Ok(None),
        }
    }

    pub async fn create_profile(&self, draft: ProfileDraft) -> Result<Profile> {
        let loader_version = self.loader_version(draft.loader, &draft.game_version).await?;
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        let name = clean_name(&draft.name, &draft.game_version);
        let profile = Profile {
            id: unique_id(&name, &profiles),
            name,
            icon: draft.icon,
            game_version: draft.game_version,
            loader: draft.loader,
            loader_version,
            memory_mb: draft.memory_mb.map(clamp_memory),
            last_played: None,
        };
        tokio::fs::create_dir_all(self.paths.instance(&profile.id).join("mods")).await?;
        profiles.push(profile.clone());
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        Ok(profile)
    }

    /// Changes a profile. The folder (and so the id) stays the same; a new
    /// game version or loader picks the newest matching loader build.
    pub async fn update_profile(&self, id: &str, draft: ProfileDraft) -> Result<Profile> {
        let current = self.profile(id).await?;
        let loader_version = if current.game_version == draft.game_version && current.loader == draft.loader {
            current.loader_version
        } else {
            self.loader_version(draft.loader, &draft.game_version).await?
        };
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        let profile = profiles
            .iter_mut()
            .find(|profile| profile.id == id)
            .ok_or_else(|| Error::other(format!("profile {id} not found")))?;
        profile.name = clean_name(&draft.name, &draft.game_version);
        profile.icon = draft.icon;
        profile.game_version = draft.game_version;
        profile.loader = draft.loader;
        profile.loader_version = loader_version;
        profile.memory_mb = draft.memory_mb.map(clamp_memory);
        let updated = profile.clone();
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        Ok(updated)
    }

    /// Removes the profile together with its game directory (worlds included).
    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        profiles.retain(|profile| profile.id != id);
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        match tokio::fs::remove_dir_all(self.paths.instance(id)).await {
            Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err.into()),
            _ => Ok(()),
        }
    }

    /// Release versions, newest first, flagged with Fabric support.
    pub async fn game_versions(&self) -> Result<Vec<GameVersion>> {
        let (manifest, fabric) =
            tokio::join!(mojang::manifest(&self.http, &self.paths), fabric::game_versions(&self.http));
        let fabric: HashSet<String> =
            fabric.map(|versions| versions.into_iter().map(|v| v.version).collect()).unwrap_or_default();
        Ok(manifest?
            .versions
            .into_iter()
            .filter(|version| version.kind == "release")
            .map(|version| GameVersion {
                fabric: fabric.contains(&version.id),
                id: version.id,
                release_time: version.release_time,
            })
            .collect())
    }

    pub async fn prepare(&self, profile_id: &str, progress: Progress) -> Result<LaunchPlan> {
        let settings = self.settings().await?;
        let profile = self.profile(profile_id).await?;
        let account = Account::offline(&settings.username);
        let memory_mb = profile.memory_mb.unwrap_or(settings.memory_mb);
        let request = LaunchRequest { profile: &profile, account: &account, memory_mb };
        let plan = launch::prepare(&self.http, &self.paths, &request, &progress).await?;
        self.mark_played(profile_id).await?;
        Ok(plan)
    }

    async fn mark_played(&self, id: &str) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        for profile in profiles.iter_mut().filter(|profile| profile.id == id) {
            profile.last_played = Some(unix_now());
        }
        store::write_json(&self.paths.profiles_file(), &profiles).await
    }

    /// Saves in the profile, most recently played first.
    pub async fn worlds(&self, profile_id: &str) -> Result<Vec<World>> {
        let saves = self.paths.instance(profile_id).join("saves");
        let mut entries = match tokio::fs::read_dir(&saves).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err.into()),
        };
        let mut worlds = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let level = entry.path().join("level.dat");
            let last_played = tokio::fs::metadata(&level).await.ok().and_then(|meta| meta.modified().ok()).map(unix);
            worlds.push(World { name: entry.file_name().to_string_lossy().into_owned(), last_played });
        }
        worlds.sort_by_key(|world| std::cmp::Reverse(world.last_played));
        Ok(worlds)
    }

    fn mod_index(&self, profile_id: &str) -> PathBuf {
        self.paths.instance(profile_id).join("vega-mods.json")
    }

    fn mods_dir(&self, profile_id: &str) -> PathBuf {
        self.paths.instance(profile_id).join("mods")
    }

    pub async fn installed_mods(&self, profile_id: &str) -> Result<Vec<InstalledMod>> {
        store::read_json(&self.mod_index(profile_id)).await
    }

    pub async fn search_mods(&self, profile_id: &str, query: &str) -> Result<Vec<ModResult>> {
        let profile = self.profile(profile_id).await?;
        let installed = self.installed_mods(profile_id).await?;
        let hits = modrinth::search(&self.http, query, &profile.game_version).await?;
        Ok(hits
            .into_iter()
            .map(|hit| ModResult { installed: installed.iter().any(|m| m.project_id == hit.project_id), hit })
            .collect())
    }

    /// Installs a mod and its required dependencies; returns the new mod list.
    pub async fn install_mod(&self, profile_id: &str, project_id: &str) -> Result<Vec<InstalledMod>> {
        let profile = self.profile(profile_id).await?;
        if profile.loader == Loader::Vanilla {
            return Err(Error::other("vanilla profiles cannot load mods, switch the profile to Fabric"));
        }
        let _guard = self.write_lock.lock().await;
        let mut installed = self.installed_mods(profile_id).await?;
        let mods_dir = self.mods_dir(profile_id);
        let result = modrinth::install(&self.http, &mods_dir, &profile.game_version, project_id, &mut installed).await;
        // Keep track of whatever made it to disk, even if a later dependency failed.
        store::write_json(&self.mod_index(profile_id), &installed).await?;
        result.map(|_| installed)
    }

    /// Disabling renames `x.jar` to `x.jar.disabled`, which Fabric skips.
    pub async fn set_mod_enabled(
        &self,
        profile_id: &str,
        project_id: &str,
        enabled: bool,
    ) -> Result<Vec<InstalledMod>> {
        let _guard = self.write_lock.lock().await;
        let mut installed = self.installed_mods(profile_id).await?;
        let mods_dir = self.mods_dir(profile_id);
        if let Some(entry) = installed.iter_mut().find(|m| m.project_id == project_id)
            && entry.enabled != enabled
        {
            let jar = mods_dir.join(&entry.filename);
            let disabled = disabled_path(&mods_dir, &entry.filename);
            let (from, to) = if enabled { (disabled, jar) } else { (jar, disabled) };
            tokio::fs::rename(from, to).await?;
            entry.enabled = enabled;
        }
        store::write_json(&self.mod_index(profile_id), &installed).await?;
        Ok(installed)
    }

    pub async fn remove_mod(&self, profile_id: &str, project_id: &str) -> Result<Vec<InstalledMod>> {
        let _guard = self.write_lock.lock().await;
        let mut installed = self.installed_mods(profile_id).await?;
        if let Some(pos) = installed.iter().position(|m| m.project_id == project_id) {
            let removed = installed.remove(pos);
            let mods_dir = self.mods_dir(profile_id);
            let file = if removed.enabled {
                mods_dir.join(&removed.filename)
            } else {
                disabled_path(&mods_dir, &removed.filename)
            };
            if let Err(err) = tokio::fs::remove_file(file).await
                && err.kind() != std::io::ErrorKind::NotFound
            {
                return Err(err.into());
            }
        }
        store::write_json(&self.mod_index(profile_id), &installed).await?;
        Ok(installed)
    }
}

fn disabled_path(mods_dir: &Path, filename: &str) -> PathBuf {
    mods_dir.join(format!("{filename}.disabled"))
}

fn clean_name(name: &str, game_version: &str) -> String {
    match name.trim() {
        "" => game_version.to_owned(),
        name => name.chars().take(40).collect(),
    }
}

fn clamp_memory(mb: u32) -> u32 {
    mb.clamp(1024, 32768)
}

fn unix(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn unix_now() -> u64 {
    unix(SystemTime::now())
}

/// A folder-safe id derived from the profile name.
fn unique_id(name: &str, profiles: &[Profile]) -> String {
    let slug = name
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let base = if slug.is_empty() { "profile".to_owned() } else { slug };
    let mut id = base.clone();
    let mut n = 2;
    while profiles.iter().any(|profile| profile.id == id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(name: &str, loader: Loader) -> ProfileDraft {
        ProfileDraft { name: name.into(), icon: "sword".into(), game_version: "1.21.1".into(), loader, memory_mb: None }
    }

    #[test]
    fn profile_ids() {
        let taken = Profile {
            id: "fabric-1-21-1".into(),
            name: String::new(),
            icon: "cube".into(),
            game_version: String::new(),
            loader: Loader::Fabric,
            loader_version: None,
            memory_mb: None,
            last_played: None,
        };
        assert_eq!(unique_id("Fabric 1.21.1", &[]), "fabric-1-21-1");
        assert_eq!(unique_id("Fabric 1.21.1", &[taken]), "fabric-1-21-1-2");
        assert_eq!(unique_id("Şş!", &[]), "profile");
    }

    #[tokio::test]
    async fn vanilla_profile_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = Launcher::new(Paths::new(dir.path())).unwrap();

        let profile = launcher.create_profile(draft("  PvP  ", Loader::Vanilla)).await.unwrap();
        assert_eq!((profile.id.as_str(), profile.name.as_str(), profile.icon.as_str()), ("pvp", "PvP", "sword"));

        let mut edit = draft("PvP 1.21", Loader::Vanilla);
        edit.icon = "flame".into();
        edit.memory_mb = Some(100);
        let updated = launcher.update_profile("pvp", edit).await.unwrap();
        assert_eq!((updated.id.as_str(), updated.icon.as_str(), updated.memory_mb), ("pvp", "flame", Some(1024)));

        tokio::fs::create_dir_all(dir.path().join("instances/pvp/saves/My World")).await.unwrap();
        let worlds = launcher.worlds("pvp").await.unwrap();
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].name, "My World");

        launcher.delete_profile("pvp").await.unwrap();
        assert!(launcher.profiles().await.unwrap().is_empty());
        assert!(!dir.path().join("instances/pvp").exists());
    }

    #[tokio::test]
    async fn toggling_mods_renames_the_jar() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = Launcher::new(Paths::new(dir.path())).unwrap();
        let mods = dir.path().join("instances/p/mods");
        tokio::fs::create_dir_all(&mods).await.unwrap();
        tokio::fs::write(mods.join("sodium.jar"), b"jar").await.unwrap();
        let entry = InstalledMod {
            project_id: "AANobbMI".into(),
            version_id: "v".into(),
            version_number: "1".into(),
            title: "Sodium".into(),
            icon_url: None,
            filename: "sodium.jar".into(),
            dependency: false,
            enabled: true,
        };
        store::write_json(&launcher.mod_index("p"), &vec![entry]).await.unwrap();

        let list = launcher.set_mod_enabled("p", "AANobbMI", false).await.unwrap();
        assert!(!list[0].enabled);
        assert!(mods.join("sodium.jar.disabled").exists() && !mods.join("sodium.jar").exists());

        launcher.set_mod_enabled("p", "AANobbMI", true).await.unwrap();
        assert!(mods.join("sodium.jar").exists());

        launcher.set_mod_enabled("p", "AANobbMI", false).await.unwrap();
        assert!(launcher.remove_mod("p", "AANobbMI").await.unwrap().is_empty());
        assert!(!mods.join("sodium.jar.disabled").exists());
    }
}
