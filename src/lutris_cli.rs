use crate::lutris_db::LutrisDatabase;
use crate::lutris_util::LutrisConfig;
use crate::rustris_paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Global Lutris configuration instance
static LUTRIS_CONFIG: OnceLock<LutrisConfig> = OnceLock::new();

/// Get or initialize the global Lutris configuration
fn get_lutris_config() -> &'static LutrisConfig {
    LUTRIS_CONFIG.get_or_init(|| {
        LutrisConfig::auto_detect()
            .unwrap_or_else(|_| {
                // Fallback to system config if auto-detect fails
                // This shouldn't happen in practice since auto_detect returns an error
                // but we need a fallback for the unwrap_or_else
                LutrisConfig::system()
            })
    })
}

/// Wine architecture options
/// Note: Proton is NOT compatible with Win32 prefixes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WineArch {
    #[serde(rename = "win32")]
    Win32,
    #[serde(rename = "win64")]
    Win64,
    #[serde(rename = "auto")]
    Auto,
}

impl WineArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            WineArch::Win32 => "win32",
            WineArch::Win64 => "win64",
            WineArch::Auto => "auto",
        }
    }

}

/// Lutris installer YAML structure
#[derive(Debug, Serialize)]
struct LutrisInstaller {
    name: String,
    game_slug: String,
    version: String,
    slug: String,
    runner: String,
    script: InstallerScript,
}

#[derive(Debug, Serialize)]
struct InstallerScript {
    files: Vec<InstallerFile>,
    game: InstallerGameConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    wine: Option<InstallerWineConfig>,
    installer: Vec<InstallerTask>,
}

#[derive(Debug, Serialize)]
struct InstallerFile {
    installer: String,
}

#[derive(Debug, Serialize)]
struct InstallerGameConfig {
    exe: String,
    prefix: String,
    arch: String,
}

#[derive(Debug, Serialize)]
struct InstallerWineConfig {
    version: String,
}

#[derive(Debug, Serialize)]
struct InstallerTask {
    task: TaskDetails,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum TaskDetails {
    CreatePrefix {
        name: String,
        prefix: String,
        arch: String,
    },
    Winetricks {
        name: String,
        app: String,
        prefix: String,
        arch: String,
        description: String,
    },
    WineExec {
        name: String,
        executable: String,
        prefix: String,
        arch: String,
        description: String,
    },
}

/// Windows version presets for winetricks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowsVersion {
    #[serde(rename = "win20")]
    Win20,
    #[serde(rename = "win30")]
    Win30,
    #[serde(rename = "win31")]
    Win31,
    #[serde(rename = "nt351")]
    Nt351,
    #[serde(rename = "nt40")]
    Nt40,
    #[serde(rename = "win95")]
    Win95,
    #[serde(rename = "win98")]
    Win98,
    #[serde(rename = "winme")]
    WinMe,
    #[serde(rename = "win2k")]
    Win2k,
    #[serde(rename = "win2k3")]
    Win2k3,
    #[serde(rename = "win2k8")]
    Win2k8,
    #[serde(rename = "win2k8r2")]
    Win2k8r2,
    #[serde(rename = "winxp")]
    WinXp,
    #[serde(rename = "vista")]
    Vista,
    #[serde(rename = "win7")]
    Win7,
    #[serde(rename = "win8")]
    Win8,
    #[serde(rename = "win81")]
    Win81,
    #[serde(rename = "win10")]
    Win10,
    #[serde(rename = "win11")]
    Win11,
}

impl WindowsVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowsVersion::Win20 => "win20",
            WindowsVersion::Win30 => "win30",
            WindowsVersion::Win31 => "win31",
            WindowsVersion::Nt351 => "nt351",
            WindowsVersion::Nt40 => "nt40",
            WindowsVersion::Win95 => "win95",
            WindowsVersion::Win98 => "win98",
            WindowsVersion::WinMe => "winme",
            WindowsVersion::Win2k => "win2k",
            WindowsVersion::Win2k3 => "win2k3",
            WindowsVersion::Win2k8 => "win2k8",
            WindowsVersion::Win2k8r2 => "win2k8r2",
            WindowsVersion::WinXp => "winxp",
            WindowsVersion::Vista => "vista",
            WindowsVersion::Win7 => "win7",
            WindowsVersion::Win8 => "win8",
            WindowsVersion::Win81 => "win81",
            WindowsVersion::Win10 => "win10",
            WindowsVersion::Win11 => "win11",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WindowsVersion::Win20 => "Windows 2.0",
            WindowsVersion::Win30 => "Windows 3.0",
            WindowsVersion::Win31 => "Windows 3.1",
            WindowsVersion::Nt351 => "Windows NT 3.51",
            WindowsVersion::Nt40 => "Windows NT 4.0",
            WindowsVersion::Win95 => "Windows 95",
            WindowsVersion::Win98 => "Windows 98",
            WindowsVersion::WinMe => "Windows ME",
            WindowsVersion::Win2k => "Windows 2000",
            WindowsVersion::Win2k3 => "Windows 2003",
            WindowsVersion::Win2k8 => "Windows 2008",
            WindowsVersion::Win2k8r2 => "Windows 2008 R2",
            WindowsVersion::WinXp => "Windows XP",
            WindowsVersion::Vista => "Windows Vista",
            WindowsVersion::Win7 => "Windows 7",
            WindowsVersion::Win8 => "Windows 8",
            WindowsVersion::Win81 => "Windows 8.1",
            WindowsVersion::Win10 => "Windows 10",
            WindowsVersion::Win11 => "Windows 11",
        }
    }

}

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

/// Check if Lutris is installed and available
pub fn is_lutris_installed() -> bool {
    get_lutris_config().is_lutris_available()
}

/// List only installed games from Lutris
pub async fn list_installed_games() -> Result<Vec<LutrisGame>, String> {
    let output = get_lutris_config()
        .build_tokio_command()
        .arg("-l")  // --list-games
        .arg("-o")  // --installed (only installed games)
        .arg("-j")  // --json
        .output()
        .await
        .map_err(|e| format!("Failed to run lutris: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("   Stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!("Lutris command failed with status {:?}: {}", output.status, stderr));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in output: {}", e))?;

    if stdout.trim().is_empty() {
        println!("   Lutris returned empty output");
        println!("   This might mean:");
        println!("      - No games are installed in Lutris");
        println!("      - Lutris database is empty");
        println!("      - Try: lutris -l -j (to see all games)");
        return Ok(Vec::new());
    }

    let games: Vec<LutrisGame> = serde_json::from_str(&stdout)
        .map_err(|e| {
            println!("   JSON parse error: {}", e);
            println!("   Full output:");
            println!("{}", stdout);
            format!("Failed to parse JSON: {}. Output was {} bytes", e, stdout.len())
        })?;

    Ok(games)
}

/// Launch a game using Lutris with output capture for real-time log streaming
pub async fn launch_game_via_lutris_with_capture(
    slug: &str,
    buffer: std::sync::Arc<std::sync::Mutex<crate::game_log_buffer::LogBuffer>>,
    window: tauri::Window,
) -> Result<(), String> {
    use crate::game_log_buffer::LogStreamer;
    use std::process::Stdio;

    println!("Launching game via Lutris with log capture: {}", slug);

    let uri = format!("lutris:rungame/{}", slug);
    println!("   Running: lutris {}", uri);

    // Spawn lutris with stdout/stderr capture
    let mut child = get_lutris_config()
        .build_tokio_command()
        .arg(&uri)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    println!("   Lutris spawned with PID: {}", child.id().unwrap_or(0));

    // Get stdout and stderr handles
    let stdout = child.stdout.take()
        .ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take()
        .ok_or("Failed to capture stderr")?;

    // Start streaming logs from both stdout and stderr
    let buffer_clone = buffer.clone();
    let slug_clone = slug.to_string();
    let window_clone = window.clone();

    // Stream stdout
    let streamer_stdout = LogStreamer::new(slug_clone.clone(), buffer.clone());
    tokio::spawn(async move {
        streamer_stdout.stream_output(stdout, window_clone).await;
    });

    // Stream stderr
    let streamer_stderr = LogStreamer::new(slug_clone, buffer_clone);
    tokio::spawn(async move {
        streamer_stderr.stream_output(stderr, window).await;
    });

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
    println!("Loading games from Lutris database...");
    let db = LutrisDatabase::new()?;
    let db_games = db.get_installed_games()?;

    println!("Found {} games in database", db_games.len());

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

    println!("Loaded {} games with config data", games.len());
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
        println!("Lutris default wine (custom path): {}", custom_path);
        // Strip /proton suffix to get the folder path (to match dropdown values)
        let path = PathBuf::from(custom_path);
        if let Some(parent) = path.parent() {
            return Some(parent.to_string_lossy().to_string());
        }
        return Some(custom_path.to_string());
    }

    // If only version name is set (not custom_wine_path), resolve it by looking in Lutris proton directory
    if let Some(version_name) = wine_section.get("version").and_then(|v| v.as_str()) {
        println!("Lutris default wine (version name): {}", version_name);

        // Look for this version in Lutris proton directory
        if let Some(proton_dir) = rustris_paths::lutris_proton_dir() {
            let version_path = proton_dir.join(version_name);
            if version_path.exists() {
                println!("   Found version at: {}", version_path.display());
                return Some(version_path.to_string_lossy().to_string());
            } else {
                println!("   Version '{}' not found in Lutris proton directory", version_name);
            }
        }
    }

    None
}

/// Set Lutris's default Wine version in runners/wine.yml
pub fn set_lutris_default_wine_version(wine_path: &str) -> Result<(), String> {
    println!("Setting Lutris default wine version to: {}", wine_path);

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

    println!("   Lutris default wine version updated!");

    Ok(())
}

/// Update the Wine/Proton version for a specific game
pub async fn update_game_wine_version(slug: &str, wine_version: &str) -> Result<(), String> {
    println!("Updating wine version for game: {}", slug);
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

    println!("   Config file: {:?}", config_file);

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

    println!("   Wine version updated successfully!");

    Ok(())
}

/// Generate a Lutris installer YAML for a Windows .exe installer
fn generate_installer_yaml(
    exe_path: &str,
    game_name: &str,
    game_slug: &str,
    wine_version: Option<String>,
    arch: &WineArch,
    windows_version: Option<&WindowsVersion>,
) -> Result<String, String> {
    let arch_str = arch.as_str().to_string();

    // Build installer tasks
    let mut tasks = vec![
        InstallerTask {
            task: TaskDetails::CreatePrefix {
                name: "create_prefix".to_string(),
                prefix: "$GAMEDIR".to_string(),
                arch: arch_str.clone(),
            },
        },
    ];

    // Add Windows version task if specified
    if let Some(win_ver) = windows_version {
        tasks.push(InstallerTask {
            task: TaskDetails::Winetricks {
                name: "winetricks".to_string(),
                app: win_ver.as_str().to_string(),
                prefix: "$GAMEDIR".to_string(),
                arch: arch_str.clone(),
                description: format!("Setting Windows version to {}", win_ver.display_name()),
            },
        });
    }

    // Add wineexec task
    tasks.push(InstallerTask {
        task: TaskDetails::WineExec {
            name: "wineexec".to_string(),
            executable: "installer".to_string(),
            prefix: "$GAMEDIR".to_string(),
            arch: arch_str.clone(),
            description: "Installing game...".to_string(),
        },
    });

    // Extract wine version name from path if specified
    let wine_config = wine_version.and_then(|version| {
        let version_name = PathBuf::from(&version)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or(version);

        Some(InstallerWineConfig {
            version: version_name,
        })
    });

    // Build the full installer structure
    let installer = LutrisInstaller {
        name: game_name.to_string(),
        game_slug: game_slug.to_string(),
        version: "Custom Install".to_string(),
        slug: format!("{}-custom", game_slug),
        runner: "wine".to_string(),
        script: InstallerScript {
            files: vec![InstallerFile {
                installer: exe_path.to_string(),
            }],
            game: InstallerGameConfig {
                exe: "drive_c/Program Files".to_string(),
                prefix: "$GAMEDIR".to_string(),
                arch: arch_str,
            },
            wine: wine_config,
            installer: tasks,
        },
    };

    // Serialize to YAML
    serde_yaml::to_string(&installer)
        .map_err(|e| format!("Failed to serialize installer YAML: {}", e))
}

/// Run a game installer using Lutris installer system
/// Returns the game slug for the installed game
///
/// # Arguments
/// * `exe_path` - Path to the Windows .exe installer
/// * `game_name` - Name of the game
/// * `wine_version` - Optional Wine/Proton version path. If not provided, uses Lutris default.
/// * `arch` - Wine architecture (win32, win64, or auto). Defaults to win64.
///   Note: Proton is NOT compatible with win32 prefixes
/// * `windows_version` - Optional Windows version preset (e.g., win7, win10)
pub async fn run_wine_installer(
    exe_path: String,
    game_name: String,
    wine_version: Option<String>,
    arch: Option<WineArch>,
    windows_version: Option<WindowsVersion>,
) -> Result<String, String> {
    println!("Installing game via Lutris: {}", game_name);
    println!("Installer: {}", exe_path);

    // Verify the installer file exists
    let installer_path = PathBuf::from(&exe_path);
    if !installer_path.exists() {
        return Err(format!("Installer file not found: {}", exe_path));
    }

    // Use provided wine version or get Lutris's default
    let wine_version = wine_version.or_else(|| get_lutris_default_wine_version());
    if let Some(ref version) = wine_version {
        println!("Using wine version: {}", version);
    } else {
        println!("No wine version specified, Lutris will use its default");
    }

    // Generate a slug from the game name
    let game_slug = game_name
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    println!("Game slug: {}", game_slug);

    // Use provided arch or default to win64
    let arch = arch.unwrap_or(WineArch::Win64);
    println!("Architecture: {}", arch.as_str());

    // Log Windows version if specified
    if let Some(ref win_ver) = windows_version {
        println!("Windows version: {}", win_ver.display_name());
    }

    // Generate the installer YAML
    let installer_yaml = generate_installer_yaml(
        &exe_path,
        &game_name,
        &game_slug,
        wine_version,
        &arch,
        windows_version.as_ref(),
    )?;

    // Create a temporary YAML file
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join(format!("{}-installer.yml", game_slug));

    fs::write(&yaml_path, installer_yaml)
        .map_err(|e| format!("Failed to write installer YAML: {}", e))?;

    println!("Created installer script: {}", yaml_path.display());

    // Run lutris --install with the YAML file
    println!("Starting Lutris installer...");

    let child = get_lutris_config()
        .build_tokio_command()
        .arg("--install")
        .arg(&yaml_path)
        .spawn()
        .map_err(|e| format!("Failed to run lutris: {}", e))?;

    println!("   Lutris installer spawned with PID: {}", child.id().unwrap_or(0));
    println!("   Please complete the installation in the Lutris window.");
    println!("   The installer may ask you to:");
    println!("      - Select installer files");
    println!("      - Choose installation options");
    println!("      - Wait for downloads/installations to complete");

    // Note: We don't clean up the YAML file immediately since Lutris might need it
    // It will be cleaned up by the OS from /tmp eventually

    Ok(game_slug)
}

/// Run a Lutris installer from YAML content
/// This allows running installers fetched from Lutris API or custom YAML
pub async fn run_lutris_installer_from_yaml(
    yaml_content: String,
    game_name: String,
) -> Result<String, String> {
    println!("Installing game from YAML: {}", game_name);

    // Generate a slug from the game name
    let game_slug = game_name
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    println!("Game slug: {}", game_slug);

    // Create a temporary YAML file
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join(format!("{}-installer.yml", game_slug));

    fs::write(&yaml_path, &yaml_content)
        .map_err(|e| format!("Failed to write installer YAML: {}", e))?;

    println!("Created installer script: {}", yaml_path.display());

    // Run lutris --install with the YAML file
    println!("Starting Lutris installer...");

    let child = get_lutris_config()
        .build_tokio_command()
        .arg("--install")
        .arg(&yaml_path)
        .spawn()
        .map_err(|e| format!("Failed to run lutris: {}", e))?;

    println!("   Lutris installer spawned with PID: {}", child.id().unwrap_or(0));
    println!("   Please complete the installation in the Lutris window.");
    println!("   The installer may ask you to:");
    println!("      - Select installer files");
    println!("      - Choose installation options");
    println!("      - Wait for downloads/installations to complete");

    // Note: We don't clean up the YAML file immediately since Lutris might need it
    // It will be cleaned up by the OS from /tmp eventually

    Ok(game_slug)
}