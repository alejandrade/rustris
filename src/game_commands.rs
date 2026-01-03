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
    use std::io::Write;

    // Get system info
    let system_info = crate::get_system_info();

    // Get log content from buffer or file
    let log_content = get_game_log(slug.clone()).unwrap_or_else(|_| "No log content available".to_string());

    // Create diagnosis file content
    let mut diagnosis_content = String::new();
    diagnosis_content.push_str("=== RUSTRIS DIAGNOSIS REPORT ===\n\n");

    diagnosis_content.push_str("=== CONTACT INFORMATION ===\n\n");
    diagnosis_content.push_str("If you need help, please send this file to:\n");
    diagnosis_content.push_str("Repository: https://github.com/alejandrade/rustris\n\n");
    diagnosis_content.push_str("Reddit: https://www.reddit.com/r/Lutris/\n\n");

    diagnosis_content.push_str("Game: ");
    diagnosis_content.push_str(&slug);
    diagnosis_content.push_str("\n");
    diagnosis_content.push_str("Timestamp: ");
    diagnosis_content.push_str(&chrono::Local::now().to_rfc3339());
    diagnosis_content.push_str("\n\n");

    diagnosis_content.push_str("=== SYSTEM INFORMATION ===\n\n");
    diagnosis_content.push_str(&serde_json::to_string_pretty(&system_info).unwrap_or_else(|_| "Failed to serialize system info".to_string()));
    diagnosis_content.push_str("\n\n");
    // Get Lutris game config
    let lutris_config_content = match crate::lutris_db::LutrisDatabase::new() {
        Ok(db) => {
            match db.get_configpath(&slug) {
                Ok(configpath) => {
                    match rustris_paths::lutris_game_config(&configpath) {
                        Some(path) if path.exists() => {
                            std::fs::read_to_string(path)
                                .unwrap_or_else(|e| format!("Failed to read Lutris config: {}", e))
                        },
                        _ => "Lutris game config file not found at expected path.".to_string(),
                    }
                },
                Err(e) => format!("Could not find configpath for game '{}' in database: {}", slug, e),
            }
        },
        Err(e) => format!("Failed to open Lutris database: {}", e),
    };

    diagnosis_content.push_str("=== LUTRIS GAME CONFIGURATION ===\n\n");
    diagnosis_content.push_str(&lutris_config_content);
    diagnosis_content.push_str("\n\n");

    diagnosis_content.push_str("=== GAME LOG ===\n\n");
    diagnosis_content.push_str(&log_content);

    // Save to Downloads
    let downloads_dir = rustris_paths::downloads_dir()
        .ok_or("Could not find Downloads directory")?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let dest_filename = format!("{}_diagnosis_{}.txt", slug, timestamp);
    let dest_path = downloads_dir.join(&dest_filename);

    let mut file = std::fs::File::create(&dest_path)
        .map_err(|e| format!("Failed to create diagnosis file: {}", e))?;

    file.write_all(diagnosis_content.as_bytes())
        .map_err(|e| format!("Failed to write diagnosis file: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

/// Clear and remove the log buffer for a game
#[tauri::command]
pub fn clear_game_log(slug: String) -> Result<(), String> {
    let log_buffers = get_log_buffers();
    log_buffers.remove(&slug);
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct GameRunningStatus {
    pub is_running: bool,
    pub pids: Vec<String>,
}

#[tauri::command]
pub async fn check_game_running(slug: String) -> Result<GameRunningStatus, String> {
    // Check if the game is running by looking for lutris running this specific game
    use std::process::Command;

    println!("Checking if game is running: {}", slug);

    // Check if lutris itself is running this game
    // This is the most reliable method since Lutris manages the game process
    let pattern = format!("lutris.*{}", slug);
    println!("  pgrep pattern: {}", pattern);

    let lutris_check = Command::new("pgrep")
        .arg("-f")
        .arg(&pattern)
        .output()
        .map_err(|e| format!("Failed to run pgrep: {}", e))?;

    let is_running = lutris_check.status.success();

    if is_running {
        let stdout = String::from_utf8_lossy(&lutris_check.stdout);
        let pids: Vec<String> = stdout
            .trim()
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        println!("  RUNNING - Found {} process(es)", pids.len());

        // Show the actual command lines of matched processes
        for pid in &pids {
            if let Ok(output) = Command::new("ps")
                .arg("-p")
                .arg(pid)
                .arg("-o")
                .arg("cmd=")
                .output()
            {
                let cmd = String::from_utf8_lossy(&output.stdout);
                println!("    PID {}: {}", pid, cmd.trim());
            }
        }

        Ok(GameRunningStatus {
            is_running: true,
            pids,
        })
    } else {
        println!("  NOT RUNNING - No matching processes");
        Ok(GameRunningStatus {
            is_running: false,
            pids: vec![],
        })
    }
}

#[tauri::command]
pub fn force_close_game(pids: Vec<String>) -> Result<(), String> {
    use std::process::Command;

    println!("Force closing game processes: {:?}", pids);

    for pid in pids {
        println!("  Killing PID: {}", pid);

        // Try SIGTERM first (graceful)
        let result = Command::new("kill")
            .arg(&pid)
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("    Successfully sent SIGTERM to PID {}", pid);
            }
            Ok(_) => {
                // If SIGTERM failed, try SIGKILL (force)
                println!("    SIGTERM failed, trying SIGKILL for PID {}", pid);
                let kill_result = Command::new("kill")
                    .arg("-9")
                    .arg(&pid)
                    .output();

                if let Ok(output) = kill_result {
                    if output.status.success() {
                        println!("    Successfully sent SIGKILL to PID {}", pid);
                    } else {
                        println!("    Failed to kill PID {}", pid);
                    }
                }
            }
            Err(e) => {
                println!("    Error killing PID {}: {}", pid, e);
            }
        }
    }

    Ok(())
}

