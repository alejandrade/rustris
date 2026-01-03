use crate::lutris_cli::{self, GameData};
use crate::rustris_paths;
use crate::game_log_buffer::LogBufferManager;
use std::sync::OnceLock;
use sysinfo::{Pid, System};

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

    // Close Lutris GUI first to prevent it from capturing the debug stream
    println!("Closing Lutris GUI...");

    // Use sysinfo to find and kill Lutris processes
    let mut sys = System::new_all();
    sys.refresh_all();

    // Kill all Lutris GUI processes
    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy();
        if name == "lutris" {
            println!("Killing Lutris process (PID: {})", pid);
            process.kill();
        }
    }

    // Wait for Lutris to actually close (poll until it's gone)
    let max_wait = 50; // 5 seconds max
    for i in 0..max_wait {
        sys.refresh_all();

        let lutris_running = sys.processes().iter().any(|(_, process)| {
            process.name().to_string_lossy() == "lutris"
        });

        if !lutris_running {
            println!("Lutris GUI closed successfully");
            break;
        }

        if i == max_wait - 1 {
            println!("Warning: Lutris GUI may still be running");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

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

#[derive(serde::Serialize)]
struct DiagnosisReport {
    rustris: AppInfo,
    contact: ContactInfo,
    diagnosis: DiagnosisData,
}

#[derive(serde::Serialize)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
}

#[derive(serde::Serialize)]
struct ContactInfo {
    note: String,
    repository: String,
    reddit: String,
}

#[derive(serde::Serialize)]
struct DiagnosisData {
    game_slug: String,
    timestamp: String,
    system_info: serde_json::Value,
    lutris_configs: LutrisConfigs,
    // game_log will be manually added to avoid escaped newlines
}

#[derive(serde::Serialize)]
struct LutrisConfigs {
    game_config: Option<serde_yaml::Value>,
    runner_config: Option<serde_yaml::Value>,
}

#[tauri::command]
pub async fn save_game_log(slug: String) -> Result<String, String> {
    use std::io::Write;

    // Get system info
    let system_info = crate::get_system_info();

    // Get log content from buffer or file
    let log_content = get_game_log(slug.clone()).unwrap_or_else(|_| "No log content available".to_string());

    // Get game data from CLI to find the runner
    let game_data = lutris_cli::list_games_with_data()
        .await
        .ok()
        .and_then(|games| games.into_iter().find(|g| g.slug == slug));

    // Get Lutris configs (game and runner) and parse as YAML
    let (game_config_content, runner_config_content) = match crate::lutris_db::LutrisDatabase::new() {
        Ok(db) => {
            // Get game config
            let game_config = match db.get_configpath(&slug) {
                Ok(configpath) => {
                    match rustris_paths::lutris_game_config(&configpath) {
                        Some(path) if path.exists() => {
                            std::fs::read_to_string(path)
                                .ok()
                                .and_then(|s| serde_yaml::from_str(&s).ok())
                        },
                        _ => None,
                    }
                },
                Err(_) => None,
            };

            // Get runner config using runner from game data
            let runner_config = game_data
                .as_ref()
                .and_then(|g| g.runner.as_ref())
                .and_then(|runner| {
                    rustris_paths::lutris_runner_config(runner)
                        .and_then(|path| {
                            if path.exists() {
                                std::fs::read_to_string(path)
                                    .ok()
                                    .and_then(|s| serde_yaml::from_str(&s).ok())
                            } else {
                                None
                            }
                        })
                });

            (game_config, runner_config)
        },
        Err(_) => (None, None),
    };

    // Build the structured report
    let report = DiagnosisReport {
        rustris: AppInfo {
            name: "Rustris".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "A Rust-based Lutris game launcher and manager".to_string(),
        },
        contact: ContactInfo {
            note: "If you need help, please send this file to:".to_string(),
            repository: "https://github.com/alejandrade/rustris".to_string(),
            reddit: "https://www.reddit.com/r/Lutris/".to_string(),
        },
        diagnosis: DiagnosisData {
            game_slug: slug.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
            system_info,
            lutris_configs: LutrisConfigs {
                game_config: game_config_content,
                runner_config: runner_config_content,
            },
        },
    };

    // Serialize to YAML (without game_log)
    let mut yaml_content = serde_yaml::to_string(&report)
        .map_err(|e| format!("Failed to serialize diagnosis report to YAML: {}", e))?;

    // Manually append game_log as a block scalar (with | for literal multiline)
    yaml_content.push_str("  game_log: |\n");
    for line in log_content.lines() {
        yaml_content.push_str("    ");
        yaml_content.push_str(line);
        yaml_content.push('\n');
    }

    // Save to Downloads
    let downloads_dir = rustris_paths::downloads_dir()
        .ok_or("Could not find Downloads directory")?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let dest_filename = format!("{}_diagnosis_{}.yml", slug, timestamp);
    let dest_path = downloads_dir.join(&dest_filename);

    let mut file = std::fs::File::create(&dest_path)
        .map_err(|e| format!("Failed to create diagnosis file: {}", e))?;

    file.write_all(yaml_content.as_bytes())
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
    println!("Checking if game is running: {}", slug);

    // Use sysinfo to check for lutris processes running this game
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut matching_pids = Vec::new();

    for (pid, process) in sys.processes() {
        // Check command line for "lutris" and the game slug
        let cmd = process.cmd();
        let cmd_string: Vec<String> = cmd.iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        let cmd_joined = cmd_string.join(" ");

        if cmd_joined.contains("lutris") && cmd_joined.contains(&slug) {
            let pid_string = pid.to_string();
            println!("  Found process - PID {}: {}", pid_string, cmd_joined);
            matching_pids.push(pid_string);
        }
    }

    if !matching_pids.is_empty() {
        println!("  RUNNING - Found {} process(es)", matching_pids.len());
        Ok(GameRunningStatus {
            is_running: true,
            pids: matching_pids,
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
    println!("Force closing game processes: {:?}", pids);

    let mut sys = System::new_all();
    sys.refresh_all();

    for pid_string in pids {
        println!("  Killing PID: {}", pid_string);

        // Parse PID string to sysinfo Pid
        if let Ok(pid_num) = pid_string.parse::<usize>() {
            let pid = Pid::from_u32(pid_num as u32);

            if let Some(process) = sys.process(pid) {
                if process.kill() {
                    println!("    Successfully killed PID {}", pid_string);
                } else {
                    println!("    Failed to kill PID {}", pid_string);
                }
            } else {
                println!("    Process {} not found", pid_string);
            }
        } else {
            println!("    Invalid PID: {}", pid_string);
        }
    }

    Ok(())
}

