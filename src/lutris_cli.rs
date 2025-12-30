use crate::lutris_db::LutrisDatabase;
use crate::rustris_paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde_json::Value;
use tokio::process::Command as TokioCommand;

/// Basic game data from Lutris CLI JSON output
/// Matches the actual format from `lutris -l -j`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LutrisGame {
    pub id: Option<i64>,
    pub slug: String,
    pub name: String,
    pub runner: Option<String>,
    pub platform: Option<String>,
    pub year: Option<i32>,
    pub directory: Option<String>,
    pub playtime: Option<String>,  // Format: "X days, HH:MM:SS.microseconds" or "HH:MM:SS"
    pub lastplayed: Option<String>, // Format: "YYYY-MM-DD HH:MM:SS"
    pub configpath: Option<String>,

    // Additional fields that might be in the output
    pub installer_slug: Option<String>,
    pub installed: Option<bool>,
}

/// Extended game data with config loaded from YAML files
/// This is what we use for launching games
#[derive(Debug, Serialize, Clone)]
pub struct GameData {
    // From Lutris database
    pub slug: String,
    pub name: String,
    pub runner: Option<String>,
    pub directory: Option<String>,
    pub playtime: i64,  // Seconds
    pub last_played: Option<String>,  // RFC3339

    // From Lutris config file
    pub executable: Option<String>,
    pub wine_version: Option<String>,
    pub wine_prefix: Option<String>,
    pub environment_vars: Option<String>,

    // UI/metadata
    pub cover_url: Option<String>,
    pub debug_output: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct LutrisConfigFile {
    game: Option<GameConfig>,
    system: Option<SystemConfig>,
    wine: Option<WineConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameConfig {
    exe: Option<String>,
    prefix: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SystemConfig {
    env: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WineConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_wine_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    battleye: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eac: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fsr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overrides: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_debug: Option<String>,
}

impl LutrisGame {
    /// Parse Lutris playtime string to seconds
    /// Format: "X days, HH:MM:SS.microseconds" or "HH:MM:SS.microseconds"
    pub fn playtime_seconds(&self) -> i64 {
        let playtime_str = match &self.playtime {
            Some(s) => s,
            None => return 0,
        };

        let mut total_seconds = 0i64;

        // Check if it contains "days"
        if playtime_str.contains("days") || playtime_str.contains("day") {
            let parts: Vec<&str> = playtime_str.split(',').collect();
            if parts.len() == 2 {
                // Parse days
                if let Some(days_str) = parts[0].trim().split_whitespace().next() {
                    if let Ok(days) = days_str.parse::<i64>() {
                        total_seconds += days * 24 * 3600;
                    }
                }
                // Parse time part
                total_seconds += Self::parse_time_to_seconds(parts[1].trim());
            }
        } else {
            // Just time, no days
            total_seconds = Self::parse_time_to_seconds(playtime_str.trim());
        }

        total_seconds
    }

    /// Parse time string "HH:MM:SS.microseconds" to seconds
    fn parse_time_to_seconds(time_str: &str) -> i64 {
        let time_parts: Vec<&str> = time_str.split('.').next().unwrap_or(time_str).split(':').collect();
        if time_parts.len() == 3 {
            let hours = time_parts[0].parse::<i64>().unwrap_or(0);
            let minutes = time_parts[1].parse::<i64>().unwrap_or(0);
            let seconds = time_parts[2].parse::<i64>().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        } else {
            0
        }
    }

    /// Parse Lutris datetime to RFC3339
    /// Format: "YYYY-MM-DD HH:MM:SS"
    pub fn last_played_rfc3339(&self) -> Option<String> {
        let lastplayed_str = self.lastplayed.as_ref()?;

        // Parse "YYYY-MM-DD HH:MM:SS" format
        use chrono::NaiveDateTime;

        NaiveDateTime::parse_from_str(lastplayed_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .and_then(|dt| {
                // Convert to UTC DateTime
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                    .to_rfc3339()
                    .into()
            })
    }

    /// Load full game data by reading Lutris config files
    pub fn to_game_data(&self) -> GameData {
        // Load config from YAML file
        let config = self.load_config();

        let mut executable = None;
        let mut wine_prefix = None;
        let mut wine_version = None;
        let mut environment_vars = None;

        if let Some(cfg) = config {
            // Extract wine version from version field
            if let Some(wine_cfg) = cfg.wine {
                if let Some(version_name) = wine_cfg.version {
                    // Resolve version name to full path by checking Lutris proton directory
                    if let Some(proton_dir) = rustris_paths::lutris_proton_dir() {
                        let version_path = proton_dir.join(&version_name);
                        if version_path.exists() {
                            wine_version = Some(version_path.to_string_lossy().to_string());
                        }
                    }
                }
            }

            // Extract game config
            if let Some(game_cfg) = cfg.game {
                // Get prefix
                let prefix = game_cfg.prefix
                    .or_else(|| self.directory.clone())
                    .unwrap_or_default();

                wine_prefix = if prefix.is_empty() { None } else { Some(prefix.clone()) };

                // Get executable path
                if let Some(exe) = game_cfg.exe {
                    let exe_path = PathBuf::from(&exe);
                    let full_exe_path = if exe_path.is_absolute() {
                        exe_path
                    } else {
                        PathBuf::from(&prefix).join(&exe)
                    };
                    executable = Some(full_exe_path.to_string_lossy().to_string());
                }
            }

            // Extract environment variables
            if let Some(system_cfg) = cfg.system {
                if let Some(env) = system_cfg.env {
                    let env_string: Vec<String> = env
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    if !env_string.is_empty() {
                        environment_vars = Some(env_string.join(";"));
                    }
                }
            }
        }

        // Find cover art
        let cover_url = self.find_cover_art();

        GameData {
            slug: self.slug.clone(),
            name: self.name.clone(),
            runner: self.runner.clone(),
            directory: self.directory.clone(),
            playtime: self.playtime_seconds(),
            last_played: self.last_played_rfc3339(),
            executable,
            wine_version,
            wine_prefix,
            environment_vars,
            cover_url,
            debug_output: false,
        }
    }

    /// Load Lutris config file
    fn load_config(&self) -> Option<LutrisConfigFile> {
        let configpath = self.configpath.as_ref()?;

        // Get Lutris config file path from utility
        let config_file = rustris_paths::lutris_game_config(configpath)?;

        if !config_file.exists() {
            return None;
        }

        let yaml_content = fs::read_to_string(&config_file).ok()?;
        serde_yaml::from_str(&yaml_content).ok()
    }

    /// Find cover art in Lutris directories
    fn find_cover_art(&self) -> Option<String> {
        rustris_paths::find_cover_art(&self.slug)
            .map(|p| p.to_string_lossy().to_string())
    }
}

/// Check if Lutris is installed and available in PATH
pub fn is_lutris_installed() -> bool {
    Command::new("which")
        .arg("lutris")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// List only installed games from Lutris
pub async fn list_installed_games() -> Result<Vec<LutrisGame>, String> {
    println!("🔍 Fetching installed games from Lutris CLI...");
    println!("   Running: lutris -l -o -j (--list-games --installed --json)");

    let output = TokioCommand::new("lutris")
        .arg("-l")  // --list-games
        .arg("-o")  // --installed (only installed games)
        .arg("-j")  // --json
        .output()
        .await
        .map_err(|e| format!("Failed to run lutris: {}", e))?;

    println!("   Exit status: {:?}", output.status);

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("   ℹ️  Stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!("Lutris command failed with status {:?}: {}", output.status, stderr));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in output: {}", e))?;

    println!("   📊 Raw output length: {} bytes", stdout.len());

    if stdout.trim().is_empty() {
        println!("   ⚠️  Lutris returned empty output");
        println!("   💡 This might mean:");
        println!("      - No games are installed in Lutris");
        println!("      - Lutris database is empty");
        println!("      - Try: lutris -l -j (to see all games)");
        return Ok(Vec::new());
    }

    // Print first 500 chars of output for debugging
    let preview_len = stdout.len().min(500);
    println!("   📄 Output preview ({} chars):", preview_len);
    println!("   {}", &stdout[..preview_len]);

    let games: Vec<LutrisGame> = serde_json::from_str(&stdout)
        .map_err(|e| {
            println!("   ❌ JSON parse error: {}", e);
            println!("   🔍 Full output:");
            println!("{}", stdout);
            format!("Failed to parse JSON: {}. Output was {} bytes", e, stdout.len())
        })?;

    println!("✅ Found {} installed games from Lutris", games.len());

    // Log first few games for debugging
    if !games.is_empty() {
        println!("   First game: name={}, slug={}",
            games[0].name,
            games[0].slug
        );
    }

    Ok(games)
}

/// Launch a game using Lutris
/// Command: lutris lutris:rungame/{slug}
pub async fn launch_game_via_lutris(slug: &str) -> Result<(), String> {
    println!("🚀 Launching game via Lutris: {}", slug);

    let uri = format!("lutris:rungame/{}", slug);
    println!("   Running: lutris {}", uri);

    // Spawn lutris and don't wait for it (games run in background)
    let child = TokioCommand::new("lutris")
        .arg(&uri)
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    println!("   ✅ Lutris spawned with PID: {}", child.id().unwrap_or(0));
    println!("   Game should start momentarily...");

    Ok(())
}

/// Load wine/config data from a Lutris config file
/// Returns (wine_version, wine_prefix, environment_vars, executable)
fn load_config_from_path(
    configpath: &str,
    directory: &Option<String>,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let config_file = match rustris_paths::lutris_game_config(configpath) {
        Some(f) => f,
        None => return (None, None, None, None),
    };

    if !config_file.exists() {
        return (None, None, None, None);
    }

    let yaml_content = match fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(_) => return (None, None, None, None),
    };

    let config: LutrisConfigFile = match serde_yaml::from_str(&yaml_content) {
        Ok(c) => c,
        Err(_) => return (None, None, None, None),
    };

    let mut wine_version = None;
    let mut wine_prefix = None;
    let mut environment_vars = None;
    let mut executable = None;

    // Extract wine version from version field
    if let Some(wine_cfg) = config.wine {
        if let Some(version_name) = wine_cfg.version {
            // Resolve version name to full path by checking Lutris proton directory
            if let Some(proton_dir) = rustris_paths::lutris_proton_dir() {
                let version_path = proton_dir.join(&version_name);
                if version_path.exists() {
                    wine_version = Some(version_path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Extract game config
    if let Some(game_cfg) = config.game {
        let prefix = game_cfg
            .prefix
            .or_else(|| directory.clone())
            .unwrap_or_default();

        wine_prefix = if prefix.is_empty() {
            None
        } else {
            Some(prefix.clone())
        };

        // Get executable path
        if let Some(exe) = game_cfg.exe {
            let exe_path = PathBuf::from(&exe);
            let full_exe_path = if exe_path.is_absolute() {
                exe_path
            } else {
                PathBuf::from(&prefix).join(&exe)
            };
            executable = Some(full_exe_path.to_string_lossy().to_string());
        }
    }

    // Extract environment variables
    if let Some(system_cfg) = config.system {
        if let Some(env) = system_cfg.env {
            let env_string: Vec<String> = env.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            if !env_string.is_empty() {
                environment_vars = Some(env_string.join(";"));
            }
        }
    }

    (wine_version, wine_prefix, environment_vars, executable)
}

/// Find cover art in Lutris directories
fn find_cover_art(slug: &str) -> Option<String> {
    rustris_paths::find_cover_art(slug)
        .map(|p| p.to_string_lossy().to_string())
}

/// Get all games with full data (includes config)
pub async fn list_games_with_data() -> Result<Vec<GameData>, String> {
    println!("🔍 Loading games from Lutris database...");
    let db = LutrisDatabase::new()?;
    let db_games = db.get_installed_games()?;

    println!("✅ Found {} games in database", db_games.len());

    let games: Vec<GameData> = db_games
        .iter()
        .filter_map(|g| {
            let slug = g.slug.as_ref()?.clone();
            let name = g.name.as_ref()?.clone();

            // Load wine/config settings from YAML file
            let (wine_version, wine_prefix, environment_vars, executable) =
                if let Some(ref configpath) = g.configpath {
                    load_config_from_path(configpath, &g.directory)
                } else {
                    (None, None, None, g.executable.clone())
                };

            // Find cover art
            let cover_url = find_cover_art(&slug);

            // Convert playtime (database stores in hours as float)
            let playtime = g.playtime.unwrap_or(0.0) as i64 * 3600;

            // Convert lastplayed timestamp to RFC3339
            let last_played = g.lastplayed.and_then(|ts| {
                use chrono::{DateTime, Utc};
                DateTime::<Utc>::from_timestamp(ts as i64, 0)
                    .map(|dt| dt.to_rfc3339())
            });

            Some(GameData {
                slug,
                name,
                runner: g.runner.clone(),
                directory: g.directory.clone(),
                playtime,
                last_played,
                executable: executable.or(g.executable.clone()),
                wine_version,
                wine_prefix,
                environment_vars,
                cover_url,
                debug_output: false,
            })
        })
        .collect();

    println!("✅ Loaded {} games with config data", games.len());
    Ok(games)
}

/// Get Lutris's default Wine version from runners/wine.yml
/// Returns the full path to the wine version directory
pub fn get_lutris_default_wine_version() -> Option<String> {
    let wine_config = rustris_paths::lutris_wine_config()?;

    if !wine_config.exists() {
        return None;
    }

    let yaml_content = fs::read_to_string(&wine_config).ok()?;
    let config: serde_yaml::Value = serde_yaml::from_str(&yaml_content).ok()?;

    let wine_section = config.get("wine")?;

    // Prefer custom_wine_path if it exists (this is the full path to wine executable)
    if let Some(custom_path) = wine_section.get("custom_wine_path").and_then(|v| v.as_str()) {
        println!("📖 Lutris default wine (custom path): {}", custom_path);
        // Strip /proton suffix to get the folder path (to match dropdown values)
        let path = PathBuf::from(custom_path);
        if let Some(parent) = path.parent() {
            return Some(parent.to_string_lossy().to_string());
        }
        return Some(custom_path.to_string());
    }

    // If only version name is set (not custom_wine_path), resolve it by looking in Lutris proton directory
    if let Some(version_name) = wine_section.get("version").and_then(|v| v.as_str()) {
        println!("📖 Lutris default wine (version name): {}", version_name);

        // Look for this version in Lutris proton directory
        if let Some(proton_dir) = rustris_paths::lutris_proton_dir() {
            let version_path = proton_dir.join(version_name);
            if version_path.exists() {
                println!("   ✅ Found version at: {}", version_path.display());
                return Some(version_path.to_string_lossy().to_string());
            } else {
                println!("   ⚠️  Version '{}' not found in Lutris proton directory", version_name);
            }
        }
    }

    None
}

/// Set Lutris's default Wine version in runners/wine.yml
pub fn set_lutris_default_wine_version(wine_path: &str) -> Result<(), String> {
    println!("🍷 Setting Lutris default wine version to: {}", wine_path);

    let wine_config = rustris_paths::lutris_wine_config()
        .ok_or("Could not get wine config path")?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = wine_config.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create runners directory: {}", e))?;
    }

    // Load existing config or create new one
    let mut config: serde_yaml::Value = if wine_config.exists() {
        let yaml_content = fs::read_to_string(&wine_config)
            .map_err(|e| format!("Failed to read wine.yml: {}", e))?;
        serde_yaml::from_str(&yaml_content)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };

    // Get or create wine section
    let wine_section = config
        .as_mapping_mut()
        .ok_or("Invalid YAML structure")?
        .entry(serde_yaml::Value::String("wine".to_string()))
        .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    // Append /proton to the path (wine executable name)
    let wine_executable_path = PathBuf::from(wine_path).join("proton");
    let wine_executable_str = wine_executable_path.to_string_lossy().to_string();

    println!("   Wine executable path: {}", wine_executable_str);

    // Set custom_wine_path
    wine_section
        .as_mapping_mut()
        .ok_or("Wine section is not a mapping")?
        .insert(
            serde_yaml::Value::String("custom_wine_path".to_string()),
            serde_yaml::Value::String(wine_executable_str),
        );

    // Write back to file
    let updated_yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize YAML: {}", e))?;

    fs::write(&wine_config, updated_yaml)
        .map_err(|e| format!("Failed to write wine.yml: {}", e))?;

    println!("   ✅ Lutris default wine version updated!");

    Ok(())
}

/// Update the Wine/Proton version for a specific game
pub async fn update_game_wine_version(slug: &str, wine_version: &str) -> Result<(), String> {
    println!("🍷 Updating wine version for game: {}", slug);
    println!("   New version: {}", wine_version);

    // Get config path from Lutris database
    let db = LutrisDatabase::new()?;
    let configpath = db.get_configpath(slug)?;

    // Get full path to config file using utility
    let config_file = rustris_paths::lutris_game_config(&configpath)
        .ok_or("Could not get game config path")?;

    if !config_file.exists() {
        return Err(format!("Config file does not exist: {:?}", config_file));
    }

    println!("   📄 Config file: {:?}", config_file);

    // Load existing config
    let yaml_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut config: LutrisConfigFile = serde_yaml::from_str(&yaml_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Extract version name from path (e.g., "/path/to/rustris-GE-Proton10-27" -> "rustris-GE-Proton10-27")
    let version_name = PathBuf::from(wine_version)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid wine version path")?
        .to_string();

    println!("   Setting version to: {}", version_name);

    // Update wine version using version field (not custom_wine_path)
    // Lutris will find it in its runners directory
    if let Some(wine_config) = &mut config.wine {
        wine_config.version = Some(version_name.clone());
        wine_config.custom_wine_path = None;  // Clear custom path
    } else {
        // Create wine section if it doesn't exist
        config.wine = Some(WineConfig {
            version: Some(version_name),
            custom_wine_path: None,
            battleye: None,
            eac: None,
            fsr: None,
            overrides: None,
            show_debug: None,
        });
    }

    // Save back to file
    let updated_yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_file, updated_yaml)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    println!("   ✅ Wine version updated successfully!");

    Ok(())
}