//! Tauri shell: exposes `vega-core` to the web UI as commands and events.

use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::{Mutex, oneshot},
};
use vega_core::{
    AccountSummary, DeviceCode, Error, GameVersion, InstalledMod, Launcher, ModResult, Paths, Profile, ProfileDraft,
    Progress, ProgressEvent, SearchHit, Settings,
};

type CmdResult<T> = Result<T, Error>;

struct AppState {
    launcher: Launcher,
    /// Profiles whose game is starting (`None`) or running (`Some`); sending
    /// on the channel stops that game.
    games: Mutex<HashMap<String, Option<oneshot::Sender<()>>>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogLine {
    profile_id: String,
    line: String,
    error: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    profile_id: String,
    #[serde(flatten)]
    event: ProgressEvent,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GamePayload {
    profile_id: String,
    code: Option<i32>,
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> CmdResult<Settings> {
    state.launcher.settings().await
}

#[tauri::command]
async fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<Settings> {
    state.launcher.save_settings(settings).await
}

#[tauri::command]
async fn list_profiles(state: State<'_, AppState>) -> CmdResult<Vec<Profile>> {
    state.launcher.profiles().await
}

#[tauri::command]
async fn create_profile(state: State<'_, AppState>, draft: ProfileDraft) -> CmdResult<Profile> {
    state.launcher.create_profile(draft).await
}

#[tauri::command]
async fn update_profile(state: State<'_, AppState>, id: String, draft: ProfileDraft) -> CmdResult<Profile> {
    state.launcher.update_profile(&id, draft).await
}

#[tauri::command]
async fn search_modpacks(state: State<'_, AppState>, query: String) -> CmdResult<Vec<SearchHit>> {
    state.launcher.search_modpacks(&query).await
}

#[tauri::command]
async fn modpack_versions(state: State<'_, AppState>, slug: String) -> CmdResult<Vec<String>> {
    state.launcher.modpack_versions(&slug).await
}

#[tauri::command]
async fn create_modpack_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    slug: String,
    game_version: String,
    name: String,
    icon: String,
) -> CmdResult<Profile> {
    let progress: Progress = Arc::new(move |event| {
        let _ = app.emit("install-progress", event);
    });
    state.launcher.create_modpack_profile(&slug, &game_version, &name, &icon, progress).await
}

/// Raw upload: the body is the `.jar`, `x-profile-id` and `x-file-name`
/// headers say where it goes.
#[tauri::command]
async fn add_mod_file(state: State<'_, AppState>, request: tauri::ipc::Request<'_>) -> CmdResult<Vec<InstalledMod>> {
    let bytes = upload_body(&request)?;
    let profile_id = upload_header(&request, "x-profile-id")?;
    let file_name = upload_header(&request, "x-file-name")?;
    state.launcher.add_local_mod(&profile_id, &file_name, bytes).await
}

fn upload_body(request: &tauri::ipc::Request<'_>) -> CmdResult<Vec<u8>> {
    match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) => Ok(bytes.clone()),
        _ => Err(Error::other("expected the file's contents")),
    }
}

/// Header values arrive percent-encoded so file names can hold any character.
fn upload_header(request: &tauri::ipc::Request<'_>, name: &str) -> CmdResult<String> {
    let value = request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::other(format!("missing {name}")))?;
    Ok(percent_decode(value))
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[tauri::command]
async fn list_accounts(state: State<'_, AppState>) -> CmdResult<Vec<AccountSummary>> {
    state.launcher.accounts().await
}

#[tauri::command]
async fn add_offline_account(state: State<'_, AppState>, username: String) -> CmdResult<Vec<AccountSummary>> {
    state.launcher.add_offline_account(&username).await
}

#[tauri::command]
async fn set_active_account(state: State<'_, AppState>, id: String) -> CmdResult<Vec<AccountSummary>> {
    state.launcher.set_active_account(&id).await
}

#[tauri::command]
async fn remove_account(state: State<'_, AppState>, id: String) -> CmdResult<Vec<AccountSummary>> {
    state.launcher.remove_account(&id).await
}

#[tauri::command]
fn microsoft_available(state: State<'_, AppState>) -> bool {
    state.launcher.microsoft_available()
}

#[tauri::command]
async fn start_microsoft_login(state: State<'_, AppState>) -> CmdResult<DeviceCode> {
    state.launcher.start_microsoft_login().await
}

#[tauri::command]
async fn finish_microsoft_login(state: State<'_, AppState>) -> CmdResult<Vec<AccountSummary>> {
    state.launcher.finish_microsoft_login().await
}

#[tauri::command]
async fn cancel_microsoft_login(state: State<'_, AppState>) -> CmdResult<()> {
    state.launcher.cancel_microsoft_login().await;
    Ok(())
}

#[tauri::command]
async fn delete_profile(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.launcher.delete_profile(&id).await
}

#[tauri::command]
async fn game_versions(state: State<'_, AppState>) -> CmdResult<Vec<GameVersion>> {
    state.launcher.game_versions().await
}

#[tauri::command]
async fn installed_mods(state: State<'_, AppState>, profile_id: String) -> CmdResult<Vec<InstalledMod>> {
    state.launcher.installed_mods(&profile_id).await
}

#[tauri::command]
async fn search_mods(state: State<'_, AppState>, profile_id: String, query: String) -> CmdResult<Vec<ModResult>> {
    state.launcher.search_mods(&profile_id, &query).await
}

#[tauri::command]
async fn install_mod(
    state: State<'_, AppState>,
    profile_id: String,
    project_id: String,
) -> CmdResult<Vec<InstalledMod>> {
    state.launcher.install_mod(&profile_id, &project_id).await
}

#[tauri::command]
async fn remove_mod(
    state: State<'_, AppState>,
    profile_id: String,
    project_id: String,
) -> CmdResult<Vec<InstalledMod>> {
    state.launcher.remove_mod(&profile_id, &project_id).await
}

#[tauri::command]
async fn set_mod_enabled(
    state: State<'_, AppState>,
    profile_id: String,
    project_id: String,
    enabled: bool,
) -> CmdResult<Vec<InstalledMod>> {
    state.launcher.set_mod_enabled(&profile_id, &project_id, enabled).await
}

#[tauri::command]
async fn open_profile_folder(state: State<'_, AppState>, profile_id: String, folder: Option<String>) -> CmdResult<()> {
    let dir = state.launcher.instance_dir(&profile_id, folder.as_deref())?;
    tokio::fs::create_dir_all(&dir).await?;
    tauri_plugin_opener::open_path(dir, None::<&str>).map_err(|err| Error::other(err.to_string()))
}

#[tauri::command]
async fn stop_game(state: State<'_, AppState>, profile_id: String) -> CmdResult<()> {
    if let Some(Some(stop)) = state.games.lock().await.get_mut(&profile_id).map(Option::take) {
        let _ = stop.send(());
    }
    Ok(())
}

/// Starts a profile's game. Several profiles can run side by side; one
/// profile runs at most once, since two games cannot share a game folder.
#[tauri::command]
async fn launch(app: AppHandle, state: State<'_, AppState>, profile_id: String) -> CmdResult<()> {
    {
        let mut games = state.games.lock().await;
        if games.contains_key(&profile_id) {
            return Err(Error::other("this profile is already running"));
        }
        games.insert(profile_id.clone(), None);
    }
    let emitter = app.clone();
    let id = profile_id.clone();
    let progress: Progress = Arc::new(move |event| {
        let _ = emitter.emit("launch-progress", ProgressPayload { profile_id: id.clone(), event });
    });
    let started = async {
        let plan = state.launcher.prepare(&profile_id, progress).await?;
        Ok::<_, Error>(plan.command().spawn()?)
    };
    let mut child = match started.await {
        Ok(child) => child,
        Err(err) => {
            state.games.lock().await.remove(&profile_id);
            return Err(err);
        }
    };

    if let Some(stdout) = child.stdout.take() {
        pipe_logs(app.clone(), profile_id.clone(), stdout, false);
    }
    if let Some(stderr) = child.stderr.take() {
        pipe_logs(app.clone(), profile_id.clone(), stderr, true);
    }
    let (stop_tx, stop_rx) = oneshot::channel();
    state.games.lock().await.insert(profile_id.clone(), Some(stop_tx));
    if state.launcher.settings().await.map(|s| s.hide_on_launch).unwrap_or(false)
        && let Some(window) = app.get_webview_window("main")
    {
        let _ = window.hide();
    }
    let _ = app.emit("game-started", GamePayload { profile_id: profile_id.clone(), code: None });

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let status = tokio::select! {
            status = child.wait() => status,
            _ = stop_rx => {
                let _ = child.kill().await;
                child.wait().await
            }
        };
        let state = app.state::<AppState>();
        let mut games = state.games.lock().await;
        games.remove(&profile_id);
        // The launcher comes back once the last game has closed.
        if games.is_empty()
            && let Some(window) = app.get_webview_window("main")
        {
            let _ = window.show();
            let _ = window.set_focus();
        }
        drop(games);
        let code = status.ok().and_then(|status| status.code());
        let _ = app.emit("game-exited", GamePayload { profile_id, code });
    });
    Ok(())
}

fn pipe_logs(app: AppHandle, profile_id: String, reader: impl AsyncRead + Unpin + Send + 'static, error: bool) {
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit("game-log", LogLine { profile_id: profile_id.clone(), line, error });
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let launcher = Launcher::new(Paths::platform_default()).expect("failed to start the launcher core");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { launcher, games: Mutex::new(HashMap::new()) })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_profiles,
            create_profile,
            update_profile,
            delete_profile,
            search_modpacks,
            modpack_versions,
            create_modpack_profile,
            add_mod_file,
            list_accounts,
            add_offline_account,
            set_active_account,
            remove_account,
            microsoft_available,
            start_microsoft_login,
            finish_microsoft_login,
            cancel_microsoft_login,
            set_mod_enabled,
            game_versions,
            installed_mods,
            search_mods,
            install_mod,
            remove_mod,
            open_profile_folder,
            launch,
            stop_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vega");
}
