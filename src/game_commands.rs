use crate::lutris_cli::{self, GameData};
use crate::rustris_paths;
use crate::game_log_buffer::LogBufferManager;
use std::sync::OnceLock;

/// Global log buffer manager instance (like Lutris's LOG_BUFFERS)
static LOG_BUFFERS: OnceLock<LogBufferManager> = OnceLock::new();

/// Get the global log buffer manager
fn get_log_buffers() -> &'static LogBufferManager {
    LOG_BUFFERS.get_or_init(|| LogBufferManager::new())
}


pub struct AppState {
    // Empty for now - may add app-level state later
}

#[tauri::command]
pub async fn get_games() -> Result<Vec<GameData>, String> {
    println!("Fetching games from Lutris...");
    let games = lutris_cli::list_games_with_data().await?;
    let total_count = games.len();

    // Filter for only wine/proton games
    let wine_games: Vec<GameData> = games
        .into_iter()
        .filter(|game| {
            game.runner
                .as_ref()
                .map(|r| {
                    let r_lower = r.to_lowercase();
                    r_lower.contains("wine") || r_lower.contains("proton")
                })
                .unwrap_or(false)
        })
        .collect();

    println!("Returning {} wine/proton games (filtered from {} total)", wine_games.len(), total_count);
    Ok(wine_games)
}

#[tauri::command]
pub async fn launch_game_by_slug(slug: String, window: tauri::Window) -> Result<(), String> {
    println!("Launching game via Lutris: {}", slug);

    // Get or create buffer for this game
    let log_buffers = get_log_buffers();
    let buffer = log_buffers.get_or_create(&slug);

    // Launch game with output capture
    lutris_cli::launch_game_via_lutris_with_capture(&slug, buffer.clone(), window).await?;

    println!("   Game launch delegated to Lutris with log capture");
    Ok(())
}

#[tauri::command]
pub fn get_game_log(slug: String) -> Result<String, String> {
    let log_buffers = get_log_buffers();

    // 1. Check in-memory buffer first
    if let Some(buffer_lock) = log_buffers.get(&slug) {
        let buf = buffer_lock.lock().unwrap();

        // If the game is running or has been run this session,
        // the buffer exists. Even if it's empty, we prefer this
        // "live" view over a potentially old file on disk.
        let content = buf.get_all();
        if !content.is_empty() {
            return Ok(content);
        }
    }

    // 2. Fallback to Disk
    // We only reach this if no in-memory buffer exists for the slug
    let log_paths = rustris_paths::find_game_log_paths();

    if log_paths.is_empty() {
        return Err("No log files found in standard locations.".to_string());
    }

    // Read the most recent log file
    // Optimization: If the file is massive, reading the whole thing into a
    // String can lag the UI. For now, we'll read it all to match your original logic.
    match std::fs::read_to_string(&log_paths[0]) {
        Ok(content) => {
            if content.is_empty() {
                Ok("Log file is empty.".to_string())
            } else {
                Ok(content)
            }
        },
        Err(e) => Err(format!("Failed to read log file: {}", e)),
    }
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

/// Clear and remove the log buffer for a game
#[tauri::command]
pub fn clear_game_log(slug: String) -> Result<(), String> {
    let log_buffers = get_log_buffers();
    log_buffers.remove(&slug);
    Ok(())
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

