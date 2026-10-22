//! Tauri shell: exposes `vega-core` to the web UI as commands and events.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::{Mutex, oneshot},
};
use vega_core::{
    Error, GameVersion, Launcher, Loader, ModResult, Paths, Profile, Progress, Settings, modrinth::InstalledMod,
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
async fn create_profile(
    state: State<'_, AppState>,
    name: String,
    game_version: String,
    loader: Loader,
) -> CmdResult<Profile> {
    state.launcher.create_profile(&name, &game_version, loader).await
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
async fn open_profile_folder(state: State<'_, AppState>, profile_id: String) -> CmdResult<()> {
    let dir = state.launcher.instance_dir(&profile_id);
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
            delete_profile,
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
