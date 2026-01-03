#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// CODING STYLE RULE: Never use emojis in println! or log statements.
// Logs should be plain text for parsing and readability in terminals.

mod artwork_commands;
mod game_commands;
mod game_log_buffer;
mod installer_commands;
mod lutris_api;
mod lutris_cli;
mod lutris_commands;
mod lutris_db;
mod lutris_util;
mod proton_commands;
mod rustris_paths;
mod utility_commands;

use artwork_commands::save_artwork;
use game_commands::{
    check_game_running, clear_game_log, force_close_game, get_game_log,
    get_games, launch_game_by_slug, save_game_log, AppState,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use installer_commands::{run_wine_installer, run_lutris_installer_from_yaml};
use lutris_api::{get_lutris_installer, get_lutris_installers, search_lutris_games};
use lutris_commands::{
    check_lutris_availability,
    get_available_wine_versions,
    get_lutris_global_default_wine_version,
    set_lutris_global_default_wine_version,
    update_game_wine_version,
};
use proton_commands::{
    delete_proton_version,
    download_ge_proton,
    fetch_ge_proton_releases,
};
use utility_commands::{
    check_for_crash_log,
    delete_crash_log,
    get_system_info,
    open_target,
    trigger_test_panic,
};
use std::env;
use std::fs;
use tauri::Manager;

fn main() {
    // Set up panic handler to capture crashes
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}:{}", location.file(), location.line(), location.column())
        } else {
            "Unknown location".to_string()
        };

        let backtrace = std::backtrace::Backtrace::force_capture();

        // Get system info
        let system_info = get_system_info();

        // Create crashes directory if it doesn't exist
        if let Some(crashes_dir) = rustris_paths::rustris_crashes_dir() {
            let _ = fs::create_dir_all(&crashes_dir);

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let log_path = crashes_dir.join(format!("crash_{}.log", timestamp));

            // Build YAML crash report
            let mut log_content = String::new();
            log_content.push_str("---\n");
            log_content.push_str("rustris_crash_report:\n");
            log_content.push_str(&format!("  timestamp: \"{}\"\n", chrono::Local::now().to_rfc3339()));
            log_content.push_str(&format!("  type: \"Rust Panic\"\n"));
            log_content.push_str(&format!("  message: \"{}\"\n", panic_message.replace("\"", "\\\"")));
            log_content.push_str(&format!("  location: \"{}\"\n\n", location));

            log_content.push_str("  contact:\n");
            log_content.push_str("    developer: \"alejandrade\"\n");
            log_content.push_str("    github: \"https://github.com/alejandrade\"\n");
            log_content.push_str("    repository: \"https://github.com/alejandrade/rustris\"\n\n");

            log_content.push_str("  system_information:\n");
            log_content.push_str(&serde_json::to_string_pretty(&system_info)
                .unwrap_or_else(|_| "    failed_to_serialize: true\n".to_string())
                .lines()
                .map(|line| format!("    {}\n", line))
                .collect::<String>());
            log_content.push_str("\n");

            log_content.push_str("  backtrace: |\n");
            for line in format!("{}", backtrace).lines() {
                log_content.push_str(&format!("    {}\n", line));
            }

            if let Err(e) = fs::write(&log_path, &log_content) {
                eprintln!("Failed to write crash log to {:?}: {}", log_path, e);
            } else {
                eprintln!("Crash log written to: {:?}", log_path);
            }
        }

        eprintln!("\nRustris crashed!");
        eprintln!("Panic: {}", panic_message);
        eprintln!("Location: {}", location);
    }));

    #[cfg(target_os = "linux")]
    {
        // 1. Disable DMABUF (The #1 cause of crashes on old Intel/NVIDIA)
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

        // 2. Disable Compositing (Prevents white screens on very old GPUs)
        // This forces a simpler, more stable rendering path.
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    }
    if !lutris_cli::is_lutris_installed() {
        panic!("Lutris is not installed! This application requires Lutris to run. Please install it from https://lutris.net");
    }

    println!("Lutris is installed");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {})
        .setup(|app| {
            let handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("CommandOrControl+Q", move |_app, _shortcut, _event| {
                    println!("Ctrl+Q pressed - closing application");
                    handle.exit(0);
                })
                .unwrap();

            // Show the window after webview is ready to avoid white screen
            let window = app.get_webview_window("main").unwrap();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                window.show().unwrap();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System info & debugging
            get_system_info,
            check_for_crash_log,
            delete_crash_log,
            trigger_test_panic,
            open_target,
            // Game management
            get_games,
            launch_game_by_slug,
            // Process & Log management
            check_game_running,
            force_close_game,
            get_game_log,
            clear_game_log,
            save_game_log,
            // Lutris commands (global defaults, game-specific, and wine version scanning)
            check_lutris_availability,
            get_lutris_global_default_wine_version,
            set_lutris_global_default_wine_version,
            update_game_wine_version,
            get_available_wine_versions,
            // Proton download and management
            fetch_ge_proton_releases,
            download_ge_proton,
            delete_proton_version,
            // Lutris API
            save_artwork,
            search_lutris_games,
            get_lutris_installers,
            get_lutris_installer,
            run_wine_installer,
            run_lutris_installer_from_yaml
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
