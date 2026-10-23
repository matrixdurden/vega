//! Tauri shell: exposes `vega-core` to the web UI as commands and events.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::{Mutex, oneshot},
};
use vega_core::{
    AccountSummary, ClientInfo, DeviceCode, Error, GameVersion, InstalledMod, Launcher, ModResult, Paths, Profile,
    ProfileDraft, Progress, SearchHit, Settings,
};

type CmdResult<T> = Result<T, Error>;

struct AppState {
    launcher: Launcher,
    /// Set while Minecraft runs; sending on it stops the game.
    stop_game: Mutex<Option<oneshot::Sender<()>>>,
}

#[derive(Clone, Serialize)]
struct LogLine {
    line: String,
    error: bool,
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
async fn list_clients(state: State<'_, AppState>) -> CmdResult<Vec<ClientInfo>> {
    Ok(state.launcher.clients().await)
}

#[tauri::command]
async fn create_client_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    client_id: String,
    game_version: String,
    icon: String,
) -> CmdResult<Profile> {
    let progress: Progress = Arc::new(move |event| {
        let _ = app.emit("install-progress", event);
    });
    state.launcher.create_client_profile(&client_id, &game_version, &icon, progress).await
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

/// Raw upload: the body is the `.mrpack` file, the `x-icon` header the icon.
#[tauri::command]
async fn import_modpack(
    app: AppHandle,
    state: State<'_, AppState>,
    request: tauri::ipc::Request<'_>,
) -> CmdResult<Profile> {
    let bytes = upload_body(&request)?;
    let icon = upload_header(&request, "x-icon")?;
    let progress: Progress = Arc::new(move |event| {
        let _ = app.emit("install-progress", event);
    });
    state.launcher.import_modpack(bytes, &icon, progress).await
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
async fn stop_game(state: State<'_, AppState>) -> CmdResult<()> {
    if let Some(stop) = state.stop_game.lock().await.take() {
        let _ = stop.send(());
    }
    Ok(())
}

#[tauri::command]
async fn launch(app: AppHandle, state: State<'_, AppState>, profile_id: String) -> CmdResult<()> {
    if state.stop_game.lock().await.is_some() {
        return Err(Error::other("Minecraft is already running"));
    }
    let emitter = app.clone();
    let progress: Progress = Arc::new(move |event| {
        let _ = emitter.emit("launch-progress", event);
    });
    let plan = state.launcher.prepare(&profile_id, progress).await?;
    let hide = state.launcher.settings().await?.hide_on_launch;

    let mut child = plan.command().spawn()?;
    if let Some(stdout) = child.stdout.take() {
        pipe_logs(app.clone(), stdout, false);
    }
    if let Some(stderr) = child.stderr.take() {
        pipe_logs(app.clone(), stderr, true);
    }
    let (stop_tx, stop_rx) = oneshot::channel();
    *state.stop_game.lock().await = Some(stop_tx);
    let window = app.get_webview_window("main");
    if hide && let Some(window) = &window {
        let _ = window.hide();
    }
    let _ = app.emit("game-started", ());

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let status = tokio::select! {
            status = child.wait() => status,
            _ = stop_rx => {
                let _ = child.kill().await;
                child.wait().await
            }
        };
        app.state::<AppState>().stop_game.lock().await.take();
        if let Some(window) = window {
            let _ = window.show();
            let _ = window.set_focus();
        }
        let _ = app.emit("game-exited", status.ok().and_then(|status| status.code()));
    });
    Ok(())
}

fn pipe_logs(app: AppHandle, reader: impl AsyncRead + Unpin + Send + 'static, error: bool) {
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit("game-log", LogLine { line, error });
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
        .manage(AppState { launcher, stop_game: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_profiles,
            create_profile,
            update_profile,
            delete_profile,
            list_clients,
            create_client_profile,
            search_modpacks,
            modpack_versions,
            create_modpack_profile,
            import_modpack,
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
