use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tokio::sync::{Mutex, oneshot};

use crate::{
    Error, Result,
    account::{self, Account, AccountKind, AccountStore, AccountSummary, StoredAccount},
    clients::{self, ClientInfo},
    fabric,
    launch::{self, LaunchPlan, LaunchRequest},
    modrinth::{self, InstalledMod, SearchHit},
    mojang, msa,
    net::{Http, Progress},
    paths::Paths,
    profile::{Loader, Profile, ProfileDraft, Settings},
    store,
};

/// Profile sub-folders the UI may open.
const OPENABLE_FOLDERS: &[&str] = &["mods", "resourcepacks", "shaderpacks", "screenshots", "saves"];
/// Uploaded icons are downscaled by the UI; this only stops abuse.
const MAX_ICON_LEN: usize = 400_000;

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
    /// Serialises writes to profiles.json, accounts.json and the mod indexes.
    write_lock: Mutex<()>,
    /// The Microsoft sign-in in progress, and how to cancel it.
    login: Mutex<(Option<msa::DeviceCode>, Option<oneshot::Sender<()>>)>,
}

impl Launcher {
    pub fn new(paths: Paths) -> Result<Self> {
        Ok(Self { http: Http::new()?, paths, write_lock: Mutex::new(()), login: Mutex::new((None, None)) })
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
        settings.memory_mb = clamp_memory(settings.memory_mb);
        store::write_json(&self.paths.settings_file(), &settings).await?;
        Ok(settings)
    }

    // Accounts

    async fn account_store(&self) -> Result<AccountStore> {
        let path = self.paths.accounts_file();
        if tokio::fs::try_exists(&path).await? {
            return store::read_json(&path).await;
        }
        // Before accounts existed the offline name lived in settings.json.
        let mut accounts = AccountStore::default();
        if let Some(name) = self.settings().await?.legacy_username.filter(|name| account::valid_username(name)) {
            accounts.upsert(offline_account(&name));
            self.save_accounts(&accounts).await?;
        }
        Ok(accounts)
    }

    async fn save_accounts(&self, accounts: &AccountStore) -> Result<()> {
        let path = self.paths.accounts_file();
        store::write_json(&path, accounts).await?;
        restrict_to_owner(&path)
    }

    pub async fn accounts(&self) -> Result<Vec<AccountSummary>> {
        Ok(self.account_store().await?.summaries())
    }

    pub async fn add_offline_account(&self, username: &str) -> Result<Vec<AccountSummary>> {
        let username = username.trim();
        if !account::valid_username(username) {
            return Err(Error::other("username must be 3-16 letters, digits or underscores"));
        }
        let _guard = self.write_lock.lock().await;
        let mut accounts = self.account_store().await?;
        accounts.upsert(offline_account(username));
        self.save_accounts(&accounts).await?;
        Ok(accounts.summaries())
    }

    pub async fn set_active_account(&self, id: &str) -> Result<Vec<AccountSummary>> {
        let _guard = self.write_lock.lock().await;
        let mut accounts = self.account_store().await?;
        if accounts.accounts.iter().any(|account| account.id == id) {
            accounts.active = Some(id.to_owned());
            self.save_accounts(&accounts).await?;
        }
        Ok(accounts.summaries())
    }

    pub async fn remove_account(&self, id: &str) -> Result<Vec<AccountSummary>> {
        let _guard = self.write_lock.lock().await;
        let mut accounts = self.account_store().await?;
        accounts.accounts.retain(|account| account.id != id);
        if accounts.active.as_deref() == Some(id) {
            accounts.active = None;
        }
        self.save_accounts(&accounts).await?;
        Ok(accounts.summaries())
    }

    pub fn microsoft_available(&self) -> bool {
        msa::client_id().is_some()
    }

    /// Step one of Microsoft sign-in: the code to enter on microsoft.com/link.
    pub async fn start_microsoft_login(&self) -> Result<msa::DeviceCode> {
        let code = msa::start_device_login(&self.http).await?;
        self.login.lock().await.0 = Some(code.clone());
        Ok(code)
    }

    /// Step two: waits for the player, then saves the account as active.
    pub async fn finish_microsoft_login(&self) -> Result<Vec<AccountSummary>> {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let code = {
            let mut login = self.login.lock().await;
            login.1 = Some(cancel_tx);
            login.0.clone().ok_or_else(|| Error::other("start the Microsoft sign-in first"))?
        };
        let account = tokio::select! {
            account = msa::finish_device_login(&self.http, &code) => account,
            _ = cancel_rx => Err(Error::other("sign-in was cancelled")),
        };
        *self.login.lock().await = (None, None);
        let account = account?;
        let _guard = self.write_lock.lock().await;
        let mut accounts = self.account_store().await?;
        accounts.upsert(account);
        self.save_accounts(&accounts).await?;
        Ok(accounts.summaries())
    }

    pub async fn cancel_microsoft_login(&self) {
        if let (_, Some(cancel)) = std::mem::take(&mut *self.login.lock().await) {
            let _ = cancel.send(());
        }
    }

    /// The active account, with a fresh token for Microsoft accounts.
    async fn launch_account(&self) -> Result<Account> {
        let accounts = self.account_store().await?;
        let active = accounts.active().cloned().ok_or_else(|| Error::other("add an account before playing"))?;
        match active.kind {
            AccountKind::Offline => Ok(Account::offline(&active.username)),
            AccountKind::Microsoft => {
                let fresh = msa::refresh(&self.http, &active).await?;
                if fresh.microsoft.as_ref().map(|s| &s.access_token)
                    != active.microsoft.as_ref().map(|s| &s.access_token)
                {
                    let _guard = self.write_lock.lock().await;
                    let mut accounts = self.account_store().await?;
                    let keep_active = accounts.active.clone();
                    accounts.upsert(fresh.clone());
                    accounts.active = keep_active;
                    self.save_accounts(&accounts).await?;
                }
                let session = fresh.microsoft.expect("refresh keeps the Microsoft session");
                Ok(Account {
                    username: fresh.username,
                    uuid: fresh.id,
                    access_token: session.access_token,
                    user_type: "msa",
                    xuid: session.xuid,
                })
            }
        }
    }

    // Profiles

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

    async fn add_profile(&self, profile: Profile) -> Result<Profile> {
        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        let profile = Profile { id: unique_id(&profile.name, &profiles), ..profile };
        tokio::fs::create_dir_all(self.paths.instance(&profile.id).join("mods")).await?;
        profiles.push(profile.clone());
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        Ok(profile)
    }

    pub async fn create_profile(&self, draft: ProfileDraft) -> Result<Profile> {
        check_icon(&draft.icon)?;
        let loader_version = self.loader_version(draft.loader, &draft.game_version).await?;
        self.add_profile(Profile {
            id: String::new(),
            name: clean_name(&draft.name, &draft.game_version),
            icon: draft.icon,
            game_version: draft.game_version,
            loader: draft.loader,
            loader_version,
            client: None,
            memory_mb: draft.memory_mb.map(clamp_memory),
            last_played: None,
        })
        .await
    }

    /// Ready-made clients and the Minecraft versions each supports.
    pub async fn clients(&self) -> Vec<ClientInfo> {
        clients::catalog(&self.http).await
    }

    /// Creates a Fabric profile with a client installed in it.
    pub async fn create_client_profile(
        &self,
        client_id: &str,
        game_version: &str,
        icon: &str,
        progress: Progress,
    ) -> Result<Profile> {
        check_icon(icon)?;
        let name = clients::name(client_id).ok_or_else(|| Error::other(format!("unknown client {client_id}")))?;
        let mut profile = self
            .add_profile(Profile {
                id: String::new(),
                name: name.to_owned(),
                icon: icon.to_owned(),
                game_version: game_version.to_owned(),
                loader: Loader::Fabric,
                loader_version: None,
                client: Some(client_id.to_owned()),
                memory_mb: None,
                last_played: None,
            })
            .await?;
        let instance = self.paths.instance(&profile.id);
        let installed = match clients::install(&self.http, client_id, game_version, &instance, &progress).await {
            Ok(installed) => installed,
            Err(err) => {
                let _ = self.delete_profile(&profile.id).await;
                return Err(err);
            }
        };
        store::write_json(&self.mod_index(&profile.id), &installed.mods).await?;

        let _guard = self.write_lock.lock().await;
        let mut profiles = self.profiles().await?;
        if let Some(saved) = profiles.iter_mut().find(|p| p.id == profile.id) {
            saved.loader_version = Some(installed.loader_version.clone());
        }
        store::write_json(&self.paths.profiles_file(), &profiles).await?;
        profile.loader_version = Some(installed.loader_version);
        Ok(profile)
    }

    /// Changes a profile. The folder (and so the id) stays the same; a new
    /// game version or loader picks the newest matching loader build.
    pub async fn update_profile(&self, id: &str, draft: ProfileDraft) -> Result<Profile> {
        check_icon(&draft.icon)?;
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
        let (settings, profile, account) =
            tokio::try_join!(self.settings(), self.profile(profile_id), self.launch_account())?;
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

    // Mods

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

fn offline_account(username: &str) -> StoredAccount {
    StoredAccount {
        id: account::offline_uuid(username),
        kind: AccountKind::Offline,
        username: username.to_owned(),
        microsoft: None,
    }
}

/// accounts.json holds Microsoft refresh tokens; keep other users out.
#[cfg(unix)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_to_owner(_: &Path) -> Result<()> {
    Ok(())
}

fn check_icon(icon: &str) -> Result<()> {
    let built_in = icon.len() <= 32 && icon.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    let uploaded = icon.starts_with("data:image/") && icon.len() <= MAX_ICON_LEN;
    if built_in || uploaded { Ok(()) } else { Err(Error::other("the profile icon is not a supported image")) }
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

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
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
        assert_eq!(unique_id("Fabric 1.21.1", &[]), "fabric-1-21-1");
        assert_eq!(unique_id("Şş!", &[]), "profile");
    }

    #[test]
    fn icons() {
        assert!(check_icon("diamond").is_ok());
        assert!(check_icon("data:image/png;base64,AAAA").is_ok());
        assert!(check_icon("javascript:alert(1)").is_err());
        assert!(check_icon(&format!("data:image/png;base64,{}", "A".repeat(MAX_ICON_LEN))).is_err());
    }

    #[tokio::test]
    async fn vanilla_profile_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = Launcher::new(Paths::new(dir.path())).unwrap();

        let profile = launcher.create_profile(draft("  PvP  ", Loader::Vanilla)).await.unwrap();
        assert_eq!((profile.id.as_str(), profile.name.as_str(), profile.icon.as_str()), ("pvp", "PvP", "sword"));
        let second = launcher.create_profile(draft("PvP", Loader::Vanilla)).await.unwrap();
        assert_eq!(second.id, "pvp-2");

        let mut edit = draft("PvP 1.21", Loader::Vanilla);
        edit.icon = "tnt".into();
        edit.memory_mb = Some(100);
        let updated = launcher.update_profile("pvp", edit).await.unwrap();
        assert_eq!((updated.id.as_str(), updated.icon.as_str(), updated.memory_mb), ("pvp", "tnt", Some(1024)));

        launcher.delete_profile("pvp").await.unwrap();
        assert_eq!(launcher.profiles().await.unwrap().len(), 1);
        assert!(!dir.path().join("instances/pvp").exists());
    }

    #[tokio::test]
    async fn accounts_and_migration() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(dir.path().join("settings.json"), br#"{"username":"Steve","memoryMb":2048}"#).await.unwrap();
        let launcher = Launcher::new(Paths::new(dir.path())).unwrap();

        let accounts = launcher.accounts().await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!((accounts[0].username.as_str(), accounts[0].active), ("Steve", true));

        let accounts = launcher.add_offline_account("Alex").await.unwrap();
        assert!(accounts.iter().any(|a| a.username == "Alex" && a.active));
        assert!(launcher.add_offline_account("no spaces").await.is_err());

        let steve = accounts.iter().find(|a| a.username == "Steve").unwrap().id.clone();
        launcher.set_active_account(&steve).await.unwrap();
        assert_eq!(launcher.launch_account().await.unwrap().username, "Steve");

        let accounts = launcher.remove_account(&steve).await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert!(accounts[0].active);
        // The legacy name is not written back, so it cannot re-migrate.
        let saved = tokio::fs::read_to_string(dir.path().join("settings.json")).await.unwrap();
        launcher.save_settings(launcher.settings().await.unwrap()).await.unwrap();
        let resaved = tokio::fs::read_to_string(dir.path().join("settings.json")).await.unwrap();
        assert!(saved.contains("Steve") && !resaved.contains("Steve"));
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
