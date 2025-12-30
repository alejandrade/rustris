use serde_json::Value;

#[tauri::command]
pub async fn search_lutris_games(search: Option<String>) -> Result<Value, String> {
    let base_url = "https://lutris.net/api/games";

    let mut url = reqwest::Url::parse(base_url)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    if let Some(search_term) = search {
        url.query_pairs_mut().append_pair("search", &search_term);
    }

    let response = reqwest::get(url.as_str())
        .await
        .map_err(|e| format!("Failed to fetch from Lutris API: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Lutris API returned status: {}", response.status()));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(data)
}

#[tauri::command]
pub async fn get_lutris_installers(game_slug: String) -> Result<Value, String> {
    let url = format!("https://lutris.net/api/games/{}/installers", game_slug);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch installers: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Lutris API returned status: {}", response.status()));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(data)
}

#[tauri::command]
pub async fn get_lutris_installer(installer_id: i64) -> Result<Value, String> {
    let url = format!("https://lutris.net/api/installers/{}", installer_id);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch installer: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Lutris API returned status: {}", response.status()));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(data)
}