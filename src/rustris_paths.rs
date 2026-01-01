/// Rustris paths utility - centralized directory and file path management
use std::path::PathBuf;

// ============================================================================
// Base Directories
// ============================================================================

/// Get the Lutris data directory
/// Returns: ~/.local/share/lutris
pub fn lutris_data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("lutris"))
}

/// Get the home directory
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// Get the downloads directory
pub fn downloads_dir() -> Option<PathBuf> {
    dirs::download_dir()
}

// ============================================================================
// Lutris Subdirectories
// ============================================================================

/// Get the Lutris games config directory
/// Returns: ~/.local/share/lutris/games
pub fn lutris_games_dir() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("games"))
}

/// Get the Lutris runners directory
/// Returns: ~/.local/share/lutris/runners
pub fn lutris_runners_dir() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("runners"))
}

/// Get the Lutris wine runners directory
/// Returns: ~/.local/share/lutris/runners/wine
pub fn lutris_wine_dir() -> Option<PathBuf> {
    lutris_runners_dir().map(|d| d.join("wine"))
}

/// Get the Lutris proton runners directory
/// Returns: ~/.local/share/lutris/runners/proton
pub fn lutris_proton_dir() -> Option<PathBuf> {
    lutris_runners_dir().map(|d| d.join("proton"))
}

/// Get the Lutris coverart directory
/// Returns: ~/.local/share/lutris/coverart
pub fn lutris_coverart_dir() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("coverart"))
}

/// Get the Lutris banners directory
/// Returns: ~/.local/share/lutris/banners
pub fn lutris_banners_dir() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("banners"))
}

/// Get the Lutris icons directory
/// Returns: ~/.local/share/lutris/icons
pub fn lutris_icons_dir() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("icons"))
}

/// Get the Lutris cache directory
/// Returns: ~/.cache/lutris
pub fn lutris_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("lutris"))
}

// ============================================================================
// Specific File Paths
// ============================================================================

/// Get the Lutris wine runner config file
/// Returns: ~/.local/share/lutris/runners/wine.yml
pub fn lutris_wine_config() -> Option<PathBuf> {
    lutris_runners_dir().map(|d| d.join("wine.yml"))
}

/// Get a Lutris game config file by config name
/// Returns: ~/.local/share/lutris/games/{config_name}.yml
pub fn lutris_game_config(config_name: &str) -> Option<PathBuf> {
    lutris_games_dir().map(|d| d.join(format!("{}.yml", config_name)))
}

/// Get the Lutris main log file
/// Returns: ~/.cache/lutris/lutris.log
pub fn lutris_main_log() -> Option<PathBuf> {
    lutris_cache_dir().map(|d| d.join("lutris.log"))
}

/// Get the Lutris database file
/// Returns: ~/.local/share/lutris/pga.db
pub fn lutris_database() -> Option<PathBuf> {
    lutris_data_dir().map(|d| d.join("pga.db"))
}

// ============================================================================
// Cover Art Lookups
// ============================================================================

/// Find cover art for a game by slug
/// Searches in coverart, banners, and icons directories
/// Returns the first matching file found
pub fn find_cover_art(slug: &str) -> Option<PathBuf> {
    let extensions = vec!["jpg", "png"];

    // Try coverart directory
    if let Some(coverart_dir) = lutris_coverart_dir() {
        for ext in &extensions {
            let path = coverart_dir.join(format!("{}.{}", slug, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Try banners directory
    if let Some(banners_dir) = lutris_banners_dir() {
        for ext in &extensions {
            let path = banners_dir.join(format!("{}.{}", slug, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Try icons directory
    if let Some(icons_dir) = lutris_icons_dir() {
        for ext in &extensions {
            let path = icons_dir.join(format!("{}.{}", slug, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

// ============================================================================
// Steam/Compatibility Tools Directories
// ============================================================================

/// Get all Steam compatibility tools directories
/// Returns paths to check for Steam-installed Proton versions
pub fn steam_compat_tools_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = home_dir() {
        // Steam installed via system package
        dirs.push(home.join(".steam/root/compatibilitytools.d"));
        dirs.push(home.join(".local/share/Steam/compatibilitytools.d"));

        // Steam Flatpak
        dirs.push(home.join(".var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d"));
    }

    // Filter to only existing directories
    dirs.into_iter().filter(|d| d.exists()).collect()
}

// ============================================================================
// Log File Lookups
// ============================================================================

/// Find Proton/Wine log files
/// Returns paths to check for game logs
pub fn find_game_log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = home_dir() {
        // Proton logs (when PROTON_LOG=1)
        paths.push(home.join("steam-0.log"));

        // Legacy Lutris log location
        paths.push(home.join("lutris.log"));
    }

    // Main Lutris log
    if let Some(log) = lutris_main_log() {
        paths.push(log);
    }

    // Filter to only existing files
    paths.into_iter().filter(|p| p.exists()).collect()
}

// ============================================================================
// Wine/Proton Scanning
// ============================================================================

/// Get all wine/proton scan locations with their source labels
/// Returns: Vec<(PathBuf, &str)> where the string is the source name
pub fn wine_scan_locations() -> Vec<(PathBuf, &'static str)> {
    let mut locations = Vec::new();

    // Lutris wine/proton (includes rustris- prefixed versions)
    if let Some(wine_dir) = lutris_wine_dir() {
        locations.push((wine_dir, "Lutris"));
    }
    if let Some(proton_dir) = lutris_proton_dir() {
        locations.push((proton_dir, "Lutris"));
    }

    // Steam compatibility tools
    for steam_dir in steam_compat_tools_dirs() {
        if steam_dir.to_string_lossy().contains("flatpak") {
            locations.push((steam_dir, "Steam Flatpak"));
        } else {
            locations.push((steam_dir, "Steam"));
        }
    }

    // Filter to only existing directories
    locations.into_iter().filter(|(d, _)| d.exists()).collect()
}

/// Get system wine paths to check
pub fn system_wine_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/bin/wine"),
        PathBuf::from("/usr/local/bin/wine"),
    ]
}