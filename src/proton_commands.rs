/// GE-Proton download and management commands
use crate::rustris_paths;
use std::fs;
use tauri::Emitter;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GeProtonRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub download_url: String,
    pub size_mb: f64,
}

/// Fetch available GE-Proton releases from GitHub
#[tauri::command]
pub async fn fetch_ge_proton_releases() -> Result<Vec<GeProtonRelease>, String> {
    println!("📡 Fetching GE-Proton releases from GitHub...");

    let client = reqwest::Client::builder()
        .user_agent("Rustris")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    let releases: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut ge_releases: Vec<GeProtonRelease> = Vec::new();

    for release in releases.iter().take(5) {
        let tag_name = release["tag_name"].as_str().unwrap_or("").to_string();
        let name = release["name"].as_str().unwrap_or("").to_string();
        let published_at = release["published_at"].as_str().unwrap_or("").to_string();

        // Find the .tar.gz asset
        if let Some(assets) = release["assets"].as_array() {
            for asset in assets {
                if let Some(asset_name) = asset["name"].as_str() {
                    if asset_name.ends_with(".tar.gz") && !asset_name.ends_with(".sha512sum") {
                        let download_url = asset["browser_download_url"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let size_bytes = asset["size"].as_u64().unwrap_or(0);
                        let size_mb = size_bytes as f64 / 1024.0 / 1024.0;

                        ge_releases.push(GeProtonRelease {
                            tag_name: tag_name.clone(),
                            name: name.clone(),
                            published_at,
                            download_url,
                            size_mb,
                        });
                        break;
                    }
                }
            }
        }
    }

    println!("   Found {} GE-Proton releases", ge_releases.len());
    for release in &ge_releases {
        println!("   - {} ({:.1} MB)", release.name, release.size_mb);
    }

    Ok(ge_releases)
}

/// Download and install a GE-Proton version
#[tauri::command]
pub async fn download_ge_proton(
    tag_name: String,
    download_url: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    println!("Downloading GE-Proton: {}", tag_name);
    println!("   URL: {}", download_url);

    let proton_dir = rustris_paths::lutris_proton_dir()
        .ok_or("Could not get Lutris proton directory")?;

    // Create the proton directory if it doesn't exist
    fs::create_dir_all(&proton_dir)
        .map_err(|e| format!("Failed to create proton directory: {}", e))?;

    // Use rustris- prefix to distinguish from Lutris-managed versions
    let prefixed_name = format!("rustris-{}", tag_name);
    let installed_path = proton_dir.join(&prefixed_name);

    // First check if it exists in Lutris directory
    if installed_path.exists() {
        return Err(format!(
            "GE-Proton {} is already installed at {:?}",
            tag_name,
            installed_path
        ));
    }

    // Check if this version exists in ANY wine directory (Lutris, Steam, etc.)
    let all_versions = crate::lutris_commands::get_available_wine_versions()
        .map_err(|e| format!("Failed to check existing versions: {}", e))?;

    for version in all_versions {
        // Extract the version name without source suffix (e.g., "GE-Proton10-27 (Lutris)" -> "GE-Proton10-27")
        let version_name = version.display_name.split(" (").next().unwrap_or(&version.display_name);

        // Also check if the path ends with the tag name
        let path_ends_with_tag = version.path.ends_with(&tag_name);

        if version_name == tag_name || path_ends_with_tag {
            return Err(format!(
                "{} is already installed at: {}",
                tag_name,
                version.path
            ));
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("Rustris")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Download the tar.gz file with progress tracking
    println!("   📥 Downloading archive...");
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);

    // Stream the download and track progress
    use futures_util::StreamExt;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;
        buffer.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;

        // Emit progress event
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = app_handle.emit("download-progress", serde_json::json!({
            "tag_name": tag_name,
            "downloaded": downloaded,
            "total": total_size,
            "progress": progress,
        }));
    }

    println!("   Download complete, extracting...");

    // Emit extraction status
    let _ = app_handle.emit("download-progress", serde_json::json!({
        "tag_name": tag_name,
        "downloaded": total_size,
        "total": total_size,
        "progress": 100,
        "extracting": true,
    }));

    // Extract the tar.gz directly to the proton directory
    let decoder = flate2::read::GzDecoder::new(&buffer[..]);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(&proton_dir)
        .map_err(|e| format!("Failed to extract archive: {}", e))?;

    // Rename the extracted folder to include rustris- prefix
    let extracted_path = proton_dir.join(&tag_name);
    if extracted_path.exists() {
        fs::rename(&extracted_path, &installed_path)
            .map_err(|e| format!("Failed to rename extracted folder: {}", e))?;
        println!("   📁 Renamed to: {}", prefixed_name);
    } else {
        return Err(format!("Expected extracted folder not found: {:?}", extracted_path));
    }

    println!("   GE-Proton {} installed successfully as {}!", tag_name, prefixed_name);

    // Return the path to the installed version
    Ok(installed_path.to_string_lossy().to_string())
}

/// Delete a Proton version from wine/proton runners directories
#[tauri::command]
pub fn delete_proton_version(path: String) -> Result<(), String> {
    println!("Deleting proton version: {}", path);

    let path_buf = std::path::PathBuf::from(&path);

    if !path_buf.exists() {
        return Err("Proton version path does not exist".to_string());
    }

    // Safety check: only allow deletion from known wine/proton directories
    let path_str = path_buf.to_string_lossy();
    let allowed_paths = [
        ".local/share/lutris/runners/wine",
        ".local/share/lutris/runners/proton",
    ];

    let is_allowed = allowed_paths.iter().any(|allowed| path_str.contains(allowed));

    if !is_allowed {
        return Err("Can only delete wine/proton versions from Rustris/Lutris directories".to_string());
    }

    // Check if this is the default Lutris wine version
    if let Some(default_wine) = crate::lutris_cli::get_lutris_default_wine_version() {
        // The default wine version returns the folder path, possibly with /proton appended
        // Compare both the direct path and the path with /proton removed
        let default_path = std::path::PathBuf::from(&default_wine);
        let default_path_str = default_path.to_string_lossy();

        if path_str == default_path_str ||
           path_str == default_path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default() {
            return Err(
                "Cannot delete the default Lutris Wine/Proton version. Please set a different default first.".to_string()
            );
        }
    }

    // Delete the directory
    fs::remove_dir_all(&path_buf)
        .map_err(|e| format!("Failed to delete directory: {}", e))?;

    println!("   Deleted successfully");
    Ok(())
}