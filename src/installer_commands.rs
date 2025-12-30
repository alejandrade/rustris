use crate::lutris_cli::{self, WineArch, WindowsVersion};
use tauri::command;

#[command]
pub async fn run_wine_installer(
    exe_path: String,
    game_name: String,
    wine_version: Option<String>,
    arch: Option<WineArch>,
    windows_version: Option<WindowsVersion>,
) -> Result<String, String> {
    // Delegate to lutris_cli module which uses Lutris's installer system
    lutris_cli::run_wine_installer(exe_path, game_name, wine_version, arch, windows_version).await
}

#[command]
pub async fn run_lutris_installer_from_yaml(
    yaml_content: String,
    game_name: String,
) -> Result<String, String> {
    // Run a Lutris installer from YAML content
    lutris_cli::run_lutris_installer_from_yaml(yaml_content, game_name).await
}
