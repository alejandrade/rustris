/// Lutris domain commands - commands that interact with Lutris configuration
use crate::lutris_cli;
use crate::lutris_util::LutrisConfig;
use crate::rustris_paths;
use std::fs;

#[derive(Debug)]
struct WineVersion {
    name: String,
    source: String,
    path: std::path::PathBuf,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WineVersionInfo {
    pub path: String,
    pub display_name: String,
}

/// Get Lutris's global default wine version
/// This is the default wine version that Lutris uses for all new games
#[tauri::command]
pub fn get_lutris_global_default_wine_version() -> Option<String> {
    lutris_cli::get_lutris_default_wine_version()
}

/// Set Lutris's global default wine version
/// This sets the default wine version that Lutris uses for all new games
#[tauri::command]
pub fn set_lutris_global_default_wine_version(wine_path: String) -> Result<(), String> {
    lutris_cli::set_lutris_default_wine_version(&wine_path)
}

/// Update a specific game's wine version
/// This only affects the specified game, not the global default
#[tauri::command]
pub async fn update_game_wine_version(slug: String, wine_version: String) -> Result<(), String> {
    lutris_cli::update_game_wine_version(&slug, &wine_version).await
}

/// Get all available Wine/Proton versions from Lutris and Steam directories
#[tauri::command]
pub fn get_available_wine_versions() -> Result<Vec<WineVersionInfo>, String> {
    let mut found_versions: Vec<WineVersion> = Vec::new();

    // Get wine/proton scan locations from centralized utility
    let scan_locations = rustris_paths::wine_scan_locations();

    // Scan each location
    for (location, source) in scan_locations {
        if let Ok(entries) = std::fs::read_dir(&location) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    // Try to read version file for actual version name
                    let version_file = entry.path().join("version");
                    let version_name = if version_file.exists() {
                        // Read version file and extract version name
                        if let Ok(version_content) = fs::read_to_string(&version_file) {
                            // Version file format: "timestamp VERSION_NAME"
                            // e.g., "1762104463 GE-Proton10-25"
                            version_content
                                .split_whitespace()
                                .nth(1)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    entry.file_name().to_string_lossy().to_string()
                                })
                        } else {
                            entry.file_name().to_string_lossy().to_string()
                        }
                    } else {
                        // Fall back to directory name
                        entry.file_name().to_string_lossy().to_string()
                    };

                    found_versions.push(WineVersion {
                        name: version_name,
                        source: source.to_string(),
                        path: entry.path(),
                    });
                }
            }
        }
    }

    // Check for system wine
    let system_wine_paths = rustris_paths::system_wine_paths();

    for wine_path in system_wine_paths {
        if wine_path.exists() {
            found_versions.push(WineVersion {
                name: "System Wine".to_string(),
                source: "System".to_string(),
                path: wine_path,
            });
            break; // Only add System Wine once
        }
    }

    // Build final list with path as identifier and display name for UI
    let mut version_counts = std::collections::HashMap::new();
    for version in &found_versions {
        *version_counts.entry(&version.name).or_insert(0) += 1;
    }

    let mut wine_versions: Vec<WineVersionInfo> = found_versions
        .iter()
        .map(|v| {
            let display_name = if version_counts[&v.name] > 1 {
                // Add source identifier if there are duplicates
                format!("{} ({})", v.name, v.source)
            } else {
                v.name.clone()
            };

            WineVersionInfo {
                path: v.path.to_string_lossy().to_string(),
                display_name,
            }
        })
        .collect();

    // Sort by display name
    wine_versions.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    // Remove exact duplicates (same path)
    wine_versions.dedup_by(|a, b| a.path == b.path);

    println!("Found {} wine/proton versions", wine_versions.len());
    for version in &wine_versions {
        println!("   - {} -> {}", version.display_name, version.path);
    }

    Ok(wine_versions)
}

/// Information about Lutris installation status
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LutrisAvailability {
    pub is_available: bool,
    pub installation_type: Option<String>,
    pub install_instructions: Option<String>,
}

/// Check if Lutris is installed and available
/// Returns information about the Lutris installation or installation instructions
#[tauri::command]
pub fn check_lutris_availability() -> LutrisAvailability {
    match LutrisConfig::auto_detect() {
        Ok(config) => {
            LutrisAvailability {
                is_available: true,
                installation_type: Some(config.description()),
                install_instructions: None,
            }
        }
        Err(instructions) => {
            LutrisAvailability {
                is_available: false,
                installation_type: None,
                install_instructions: Some(instructions),
            }
        }
    }
}