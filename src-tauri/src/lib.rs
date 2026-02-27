mod commands;
mod db;

use db::connection::{DbState, open};
use db::migration;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::Manager;
use tauri::State;

const APP_ICON: &[u8] = include_bytes!("../icons/icon.png");

/// Shared state: the directory used for all app data (DB, projects, cache).
/// Resolved to `{exe_dir}/dubverse_data/` so data stays on the same drive as the installation.
pub struct DataDirState(pub PathBuf);

/// Returns `{current_exe_dir}/dubverse_data`.
fn get_app_data_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| "无法确定可执行文件所在目录".to_string())?;
    Ok(exe_dir.join("dubverse_data"))
}

#[tauri::command]
fn cmd_get_all_config(state: State<DbState>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::queries::get_all_config(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_set_config(state: State<DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::queries::set_config(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_get_provider_secret(
    state: State<DbState>,
    provider_id: String,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::queries::get_provider_secret(&conn, &provider_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_set_provider_secret(
    state: State<DbState>,
    provider_id: String,
    secret_json: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::queries::set_provider_secret(&conn, &provider_id, &secret_json).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd_get_all_config,
            cmd_set_config,
            cmd_get_provider_secret,
            cmd_set_provider_secret,
            commands::transcribe::cmd_create_project_dir,
            commands::transcribe::cmd_extract_audio,
            commands::transcribe::cmd_transcribe_elevenlabs,
            commands::transcribe::cmd_transcribe_bcut,
            commands::transcribe::cmd_save_subtitles,
        ])
        .setup(|app| {
            // Resolve data directory: {exe_dir}/dubverse_data/
            let data_dir = get_app_data_dir().expect("get app data dir");
            std::fs::create_dir_all(&data_dir).expect("create data dir");

            // Initialize database
            let conn = open(data_dir.clone()).expect("open db");
            migration::run(&conn).expect("run migrations");
            app.manage(DbState(Mutex::new(conn)));
            app.manage(DataDirState(data_dir));

            // Set window icon
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(icon) = Image::from_bytes(APP_ICON) {
                    let _ = window.set_icon(icon);
                }
            }

            // Build tray menu
            let show = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            // Build tray icon
            TrayIconBuilder::new()
                .icon(Image::from_bytes(APP_ICON)?)
                .tooltip("Dubverse")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
