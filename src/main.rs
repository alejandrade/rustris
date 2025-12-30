#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// CODING STYLE RULE: Never use emojis in println! or log statements.
// Logs should be plain text for parsing and readability in terminals.

mod artwork_commands;
mod game_commands;
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
    check_game_running, get_game_log, get_games,
    launch_game_by_slug, save_game_log, AppState,
};
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

fn main() {
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
        .manage(AppState {})
        .invoke_handler(tauri::generate_handler![
            // Game management
            get_games,
            launch_game_by_slug,
            // Process & Log management
            check_game_running,
            get_game_log,
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
