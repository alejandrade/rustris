use serde::Deserialize;
use std::env;
use std::fs;
use std::process::Command;
use sysinfo::{Disks, System};
use crate::rustris_paths;

#[tauri::command]
pub fn trigger_test_panic() {
    panic!("This is a test panic to verify crash logging works correctly");
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "camelCase")]
pub enum OpenTarget {
    Path(String),
    Url(String),
    Directory(String),
}

#[tauri::command]
pub async fn open_target(app_handle: tauri::AppHandle, target: OpenTarget) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let opener = app_handle.opener();

    match target {
        OpenTarget::Path(path) => {
            opener.open_path(path, None::<&str>).map_err(|e| e.to_string())
        },
        OpenTarget::Url(url) => {
            opener.open_url(url, None::<&str>).map_err(|e| e.to_string())
        },
        OpenTarget::Directory(path) => {
            opener.reveal_item_in_dir(path).map_err(|e| e.to_string())
        },
    }
}

#[tauri::command]
pub fn check_for_crash_log() -> Option<String> {
    if let Some(crashes_dir) = rustris_paths::rustris_crashes_dir() {
        if crashes_dir.exists() {
            // Find the most recent crash log
            if let Ok(entries) = fs::read_dir(&crashes_dir) {
                let mut crash_files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension().and_then(|s| s.to_str()) == Some("log")
                            && e.path().file_name().and_then(|s| s.to_str()).map(|s| s.starts_with("crash_")).unwrap_or(false)
                    })
                    .collect();

                // Sort by modified time, most recent first
                crash_files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
                crash_files.reverse();

                // Read the most recent crash log
                if let Some(crash_file) = crash_files.first() {
                    if let Ok(content) = fs::read_to_string(crash_file.path()) {
                        return Some(content);
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn delete_crash_log() -> Result<(), String> {
    if let Some(crashes_dir) = rustris_paths::rustris_crashes_dir() {
        if crashes_dir.exists() {
            // Find the most recent crash log
            if let Ok(entries) = fs::read_dir(&crashes_dir) {
                let mut crash_files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().extension().and_then(|s| s.to_str()) == Some("log")
                            && e.path().file_name().and_then(|s| s.to_str()).map(|s| s.starts_with("crash_")).unwrap_or(false)
                    })
                    .collect();

                // Sort by modified time, most recent first
                crash_files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
                crash_files.reverse();

                // Delete crash logs
                for file in crash_files {
                    fs::remove_file(file.path())
                        .map_err(|e| format!("Failed to delete crash log: {}", e))?;
                }

            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_system_info() -> serde_json::Value {
    let mut sys = System::new_all();
    sys.refresh_all();

    // GPU information
    let mut gpu_info = String::from("Unknown");
    let mut driver_info = String::from("Unknown");

    if let Ok(output) = Command::new("lspci").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            let gpus: Vec<&str> = stdout
                .lines()
                .filter(|line| line.contains("VGA") || line.contains("3D") || line.contains("Display"))
                .collect();
            if !gpus.is_empty() {
                gpu_info = gpus.join("; ");
            }
        }
    }

    if let Ok(contents) = fs::read_to_string("/proc/driver/nvidia/version") {
        driver_info = contents.lines().next().unwrap_or("Unknown").to_string();
    } else if let Ok(output) = Command::new("glxinfo").arg("-B").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if line.contains("OpenGL renderer") || line.contains("OpenGL version") {
                    driver_info.push_str(line.trim());
                    driver_info.push_str("; ");
                }
            }
        }
    }

    // Memory information
    let total_memory_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let available_memory_gb = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory_gb = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    // CPU information
    let cpu_count = sys.cpus().len();
    let cpu_brand = sys.cpus().first().map(|cpu| cpu.brand()).unwrap_or("Unknown");
    let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);

    // System uptime
    let uptime_secs = System::uptime();
    let uptime_hours = uptime_secs / 3600;

    // Kernel version
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    // Linux distribution info from /etc/os-release
    let mut distro_name = String::from("Unknown");
    let mut distro_version = String::from("Unknown");
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("PRETTY_NAME=") {
                distro_name = line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string();
            } else if line.starts_with("VERSION=") {
                distro_version = line.trim_start_matches("VERSION=").trim_matches('"').to_string();
            }
        }
    }

    // Desktop environment
    let desktop_env = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_|
        env::var("DESKTOP_SESSION").unwrap_or_else(|_| "Unknown".to_string())
    );

    // Display server (X11 vs Wayland)
    let display_server = if env::var("WAYLAND_DISPLAY").is_ok() {
        "Wayland"
    } else if env::var("DISPLAY").is_ok() {
        "X11"
    } else {
        "Unknown"
    };

    // WebKitGTK version
    let mut webkitgtk_version = String::from("Unknown");
    if let Ok(output) = Command::new("pkg-config").args(&["--modversion", "webkit2gtk-4.1"]).output() {
        if let Ok(version) = String::from_utf8(output.stdout) {
            webkitgtk_version = version.trim().to_string();
        }
    }

    // Disk space (root filesystem)
    let mut disk_total_gb: f64 = 0.0;
    let mut disk_available_gb: f64 = 0.0;
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        if disk.mount_point().to_str() == Some("/") {
            disk_total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            disk_available_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            break;
        }
    }

    // Current process stats
    let pid = sysinfo::get_current_pid().ok();
    let mut process_cpu = 0.0;
    let mut process_memory_mb = 0.0;
    if let Some(pid) = pid {
        if let Some(process) = sys.process(pid) {
            process_cpu = process.cpu_usage();
            process_memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
        }
    }

    serde_json::json!({
        "os": env::consts::OS,
        "arch": env::consts::ARCH,
        "family": env::consts::FAMILY,
        "app_version": env!("CARGO_PKG_VERSION"),
        "kernel_version": kernel_version,
        "distro": distro_name,
        "distro_version": distro_version,
        "desktop_environment": desktop_env,
        "display_server": display_server,
        "webkitgtk_version": webkitgtk_version,
        "gpu": gpu_info,
        "driver": driver_info,
        "cpu_brand": cpu_brand,
        "cpu_cores": cpu_count,
        "cpu_frequency_mhz": cpu_freq,
        "total_memory_gb": format!("{:.2}", total_memory_gb),
        "available_memory_gb": format!("{:.2}", available_memory_gb),
        "used_memory_gb": format!("{:.2}", used_memory_gb),
        "disk_total_gb": format!("{:.2}", disk_total_gb),
        "disk_available_gb": format!("{:.2}", disk_available_gb),
        "uptime_hours": uptime_hours,
        "process_cpu_percent": format!("{:.1}", process_cpu),
        "process_memory_mb": format!("{:.1}", process_memory_mb),
    })
}