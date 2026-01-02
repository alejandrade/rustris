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

use artwork_commands::save_artwork;
use game_commands::{
    check_game_running, clear_game_log, get_game_log,
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
use std::env;
fn main() {

    #[cfg(target_os = "linux")]
    {
        // 1. Disable DMABUF (The #1 cause of crashes on old Intel/NVIDIA)
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

        // 2. Disable Compositing (Prevents white screens on very old GPUs)
        // This forces a simpler, more stable rendering path.
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

        // 3. Force Software Rendering if necessary
        // Uncomment the line below ONLY if users still report white screens.
        // It makes the app use CPU instead of GPU for drawing.
        env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    }
    // Check if Lutris is installed
    if !lutris_cli::is_lutris_installed() {
        eprintln!("Lutris is not installed!");
        eprintln!("   This application requires Lutris to be installed.");
        eprintln!("   Please install Lutris first: https://lutris.net");
        eprintln!("   ");
        eprintln!("   On Ubuntu/Debian:");
        eprintln!("     sudo add-apt-repository ppa:lutris-team/lutris");
        eprintln!("     sudo apt update");
        eprintln!("     sudo apt install lutris");
        eprintln!("   ");
        eprintln!("   On Arch:");
        eprintln!("     sudo pacman -S lutris");
        eprintln!("   ");
        eprintln!("   On Fedora:");
        eprintln!("     sudo dnf install lutris");

        // Don't exit - allow the app to start but show error in UI
        println!("Continuing startup without Lutris...");
    } else {
        println!("Lutris is installed");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {})
        .setup(|app| {
            // Initialize log buffer cleanup task now that Tokio runtime is available
            // DISABLED: This was causing issues with log streaming
            // init_log_buffer_cleanup();

            // Register Alt+F4 to close the app (Windows convention)
            let handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("Alt+F4", move |_app, _shortcut, _event| {
                    println!("Alt+F4 pressed - closing application");
                    handle.exit(0);
                })
                .unwrap();

            // Register Ctrl+Q to close the app (Linux convention)
            let handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("CommandOrControl+Q", move |_app, _shortcut, _event| {
                    println!("Ctrl+Q pressed - closing application");
                    handle.exit(0);
                })
                .unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Game management
            get_games,
            launch_game_by_slug,
            // Process & Log management
            check_game_running,
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
