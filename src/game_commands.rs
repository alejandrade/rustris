use crate::lutris_cli::{self, GameData};
use crate::rustris_paths;

pub struct AppState {
    // Empty for now - may add app-level state later
}

#[tauri::command]
pub async fn get_games() -> Result<Vec<GameData>, String> {
    println!("🔍 Fetching games from Lutris...");
    let games = lutris_cli::list_games_with_data().await?;
    println!("✅ Returning {} games", games.len());
    Ok(games)
}

#[tauri::command]
pub async fn launch_game_by_slug(slug: String) -> Result<(), String> {
    println!("🚀 Launching game via Lutris: {}", slug);

    // Just delegate to Lutris - let it handle all the complexity
    lutris_cli::launch_game_via_lutris(&slug).await?;

    println!("   ✅ Game launch delegated to Lutris");
    Ok(())
}

#[tauri::command]
pub fn get_game_log(_slug: String) -> Result<String, String> {
    // Find game log files using utility
    let log_paths = rustris_paths::find_game_log_paths();

    if log_paths.is_empty() {
        return Err("No log files found. Try: ~/.cache/lutris/lutris.log or ~/steam-0.log".to_string());
    }

    // Read the first log file found
    std::fs::read_to_string(&log_paths[0])
        .map_err(|e| format!("Failed to read log file: {}", e))
}

#[tauri::command]
pub fn save_game_log(slug: String) -> Result<String, String> {
    // Find an existing log file
    let log_paths = rustris_paths::find_game_log_paths();
    let source_log = log_paths.first()
        .ok_or("No log file found to save")?;

    let downloads_dir = rustris_paths::downloads_dir()
        .ok_or("Could not find Downloads directory")?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let dest_filename = format!("{}_lutris_{}.log", slug, timestamp);
    let dest_path = downloads_dir.join(&dest_filename);

    std::fs::copy(source_log, &dest_path)
        .map_err(|e| format!("Failed to copy log file: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_game_running(slug: String) -> Result<bool, String> {
    // Check if the game is running by looking for lutris running this specific game
    use std::process::Command;

    // First check if lutris itself is running this game
    let lutris_check = Command::new("pgrep")
        .arg("-f")
        .arg(format!("lutris.*{}", slug))
        .output()
        .map_err(|e| format!("Failed to run pgrep: {}", e))?;

    if lutris_check.status.success() {
        return Ok(true);
    }

    // If lutris isn't running, check if the game executable is running
    let games = lutris_cli::list_installed_games().await?;
    if let Some(game) = games.into_iter().find(|g| g.slug == slug) {
        let game_data = game.to_game_data();

        // If we have an executable path, check for it
        if let Some(exe) = game_data.executable {
            // Get just the executable name without path
            if let Some(exe_name) = std::path::Path::new(&exe).file_name() {
                let exe_str = exe_name.to_string_lossy();
                let exe_check = Command::new("pgrep")
                    .arg("-f")
                    .arg(exe_str.as_ref())
                    .output()
                    .map_err(|e| format!("Failed to run pgrep: {}", e))?;

                return Ok(exe_check.status.success());
            }
        }
    }

    Ok(false)
}

