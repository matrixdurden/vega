use std::{
    collections::HashSet,
    path::PathBuf,
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
    profile::{Loader, Profile, Settings},
    store,
};

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

    pub fn instance_dir(&self, profile_id: &str) -> PathBuf {
        self.paths.instance(profile_id)
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

    pub async fn create_profile(&self, name: &str, game_version: &str, loader: Loader) -> Result<Profile> {
        let loader_version = match loader {
            Loader::Fabric => Some(fabric::latest_loader(&self.http, game_version).await?),
            Loader::Vanilla => None,
        };
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        let name = match name.trim() {
            "" => game_version.to_owned(),
            name => name.to_owned(),
        };
        let profile = Profile {
            id: unique_id(&name, &profiles),
            name,
            game_version: game_version.to_owned(),
            loader,
            loader_version,
            last_played: None,
        };
        tokio::fs::create_dir_all(self.paths.instance(&profile.id).join("mods")).await?;
        profiles.push(profile.clone());
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        Ok(profile)
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
        let request = LaunchRequest { profile: &profile, account: &account, memory_mb: settings.memory_mb };
        let plan = launch::prepare(&self.http, &self.paths, &request, &progress).await?;
        self.mark_played(profile_id).await?;
        Ok(plan)
    }

    async fn mark_played(&self, id: &str) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default();
        for profile in profiles.iter_mut().filter(|profile| profile.id == id) {
            profile.last_played = Some(now);
        }
        store::write_json(&self.paths.profiles_file(), &profiles).await
    }

    fn mod_index(&self, profile_id: &str) -> PathBuf {
        self.paths.instance(profile_id).join("vega-mods.json")
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
            return Err(Error::other("vanilla profiles cannot load mods, create a Fabric profile"));
        }
        let _guard = self.write_lock.lock().await;
        let mut installed = self.installed_mods(profile_id).await?;
        let mods_dir = self.paths.instance(profile_id).join("mods");
        let result = modrinth::install(&self.http, &mods_dir, &profile.game_version, project_id, &mut installed).await;
        // Keep track of whatever made it to disk, even if a later dependency failed.
        store::write_json(&self.mod_index(profile_id), &installed).await?;
        result.map(|_| installed)
    }

    pub async fn remove_mod(&self, profile_id: &str, project_id: &str) -> Result<Vec<InstalledMod>> {
        let _guard = self.write_lock.lock().await;
        let mut installed = self.installed_mods(profile_id).await?;
        if let Some(pos) = installed.iter().position(|m| m.project_id == project_id) {
            let removed = installed.remove(pos);
            let file = self.paths.instance(profile_id).join("mods").join(removed.filename);
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

    #[test]
    fn profile_ids() {
        let taken = Profile {
            id: "fabric-1-21-1".into(),
            name: String::new(),
            game_version: String::new(),
            loader: Loader::Fabric,
            loader_version: None,
            last_played: None,
        };
        assert_eq!(unique_id("Fabric 1.21.1", &[]), "fabric-1-21-1");
        assert_eq!(unique_id("Fabric 1.21.1", &[taken]), "fabric-1-21-1-2");
        assert_eq!(unique_id("Şş!", &[]), "profile");
    }
}
