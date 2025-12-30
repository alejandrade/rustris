use crate::lutris_cli;
use crate::rustris_paths;
use std::process::Command;
use tauri::command;

#[command]
pub async fn run_wine_installer(exe_path: String, windows_version: String) -> Result<String, String> {
    println!("🎮 Running installer: {}", exe_path);
    println!("🪟 Windows version: {}", windows_version);

    // Get Lutris's global default wine version (already a full path)
    let wine_path = lutris_cli::get_lutris_default_wine_version()
        .ok_or("No default wine version set in Lutris. Please set one in Lutris or using the Wine settings.")?;

    println!("🍷 Using wine path: {}", wine_path);

    // Find umu-run executable
    let umu_path = rustris_paths::umu_run_executable()
        .ok_or("umu-run not found. Please install Lutris which includes umu-run.")?;

    println!("🚀 Using umu-run: {}", umu_path.display());

    // Create a temporary Wine prefix for the installer
    let prefix = rustris_paths::lutris_wine_prefixes_dir()
        .ok_or("Could not get Lutris wine prefixes directory")?
        .join("installer_temp");

    std::fs::create_dir_all(&prefix).map_err(|e| e.to_string())?;
    println!("📂 Wine prefix: {}", prefix.display());

    // Build command using umu-run
    let mut cmd = Command::new(umu_path);
    cmd.arg(&exe_path);
    cmd.env("WINEPREFIX", &prefix);
    cmd.env("GAMEID", "installer");

    // Append /proton to wine_path for the executable
    let wine_executable = format!("{}/proton", wine_path);
    cmd.env("PROTONPATH", &wine_executable);

    // Set Windows version if specified
    if !windows_version.is_empty() {
        cmd.env("WINE_VERSION", &windows_version);
    }

    println!("▶️  Starting installer...");

    // Run the installer and wait for it to finish
    let status = cmd.status()
        .map_err(|e| format!("Failed to run installer: {}", e))?;

    if status.success() {
        println!("✅ Installer completed successfully");

        // Return the Program Files directory path for browsing
        let program_files = prefix.join("drive_c").join("Program Files");
        Ok(program_files.to_string_lossy().to_string())
    } else {
        Err(format!("Installer exited with code: {:?}", status.code()))
    }
}
