use std::fs;

#[tauri::command]
pub fn save_artwork(slug: String, image_data: Vec<u8>, extension: String) -> Result<String, String> {
    // Create cache directory
    let cache_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("lutris2")
        .join("coverart");

    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    // Save the image
    let cache_path = cache_dir.join(format!("{}.{}", slug, extension));
    fs::write(&cache_path, image_data)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    let path_str = cache_path.to_string_lossy().to_string();
    println!("Saved artwork for {} at: {}", slug, path_str);

    Ok(path_str)
}