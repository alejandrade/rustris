use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use tokio::process::Command as TokioCommand;

/// Type of Lutris installation being used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LutrisType {
    /// System-installed Lutris (via package manager)
    System,
    /// Flatpak Lutris
    Flatpak,
    /// Custom path specified by user
    Custom,
}

/// Information about a Linux distribution
struct DistroInfo {
    name: &'static str,
    install_command: &'static str,
}

/// Configuration for a Lutris installation
/// Automatically detects and uses system or flatpak Lutris
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LutrisConfig {
    /// Type of Lutris installation being used
    pub lutris_type: LutrisType,

    /// Path to the lutris executable
    pub executable_path: PathBuf,

    /// Lutris config directory (defaults to ~/.config/lutris)
    pub config_dir: PathBuf,

    /// Lutris data directory (defaults to ~/.local/share/lutris)
    pub data_dir: PathBuf,

    /// Lutris cache directory (defaults to ~/.cache/lutris)
    pub cache_dir: PathBuf,
}

impl LutrisConfig {
    /// Auto-detect Lutris installation
    /// Returns error if Lutris is not found
    pub fn auto_detect() -> Result<Self, String> {
        println!("Auto-detecting Lutris installation...");

        // Try system Lutris first
        if Self::is_system_lutris_available() {
            println!("Found system Lutris installation");
            return Ok(Self::system());
        }

        // Try flatpak Lutris
        if Self::is_flatpak_lutris_available() {
            println!("Found Flatpak Lutris installation");
            return Ok(Self::flatpak());
        }

        // Lutris not found - return error with installation instructions
        Err(Self::get_install_instructions())
    }

    /// Get installation instructions based on detected Linux distribution
    pub fn get_install_instructions() -> String {
        let distro = Self::detect_distro();

        format!(
            "Lutris is not installed on your system.\n\n\
            To install Lutris on {}:\n  {}\n\n\
            After installation, restart this application.",
            distro.name,
            distro.install_command
        )
    }

    /// Detect the Linux distribution
    fn detect_distro() -> DistroInfo {
        // Try to read /etc/os-release
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            if contents.contains("ID=ubuntu") || contents.contains("ID=debian") {
                return DistroInfo {
                    name: "Ubuntu/Debian",
                    install_command: "sudo apt update && sudo apt install lutris",
                };
            } else if contents.contains("ID=fedora") {
                return DistroInfo {
                    name: "Fedora",
                    install_command: "sudo dnf install lutris",
                };
            } else if contents.contains("ID=arch") || contents.contains("ID=manjaro") {
                return DistroInfo {
                    name: "Arch Linux",
                    install_command: "sudo pacman -S lutris",
                };
            } else if contents.contains("ID=opensuse") {
                return DistroInfo {
                    name: "openSUSE",
                    install_command: "sudo zypper install lutris",
                };
            }
        }

        // Default fallback
        DistroInfo {
            name: "your Linux distribution",
            install_command: "Use your package manager to install 'lutris' or visit https://lutris.net/downloads",
        }
    }

    /// Check if system Lutris is available
    fn is_system_lutris_available() -> bool {
        Command::new("which")
            .arg("lutris")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if Flatpak Lutris is available
    fn is_flatpak_lutris_available() -> bool {
        Command::new("flatpak")
            .args(&["list", "--app"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains("net.lutris.Lutris")
            })
            .unwrap_or(false)
    }

    /// Create config for system-installed Lutris
    pub fn system() -> Self {
        let home = std::env::var("HOME").expect("HOME environment variable not set");

        Self {
            lutris_type: LutrisType::System,
            executable_path: PathBuf::from("lutris"),
            config_dir: PathBuf::from(&home).join(".config/lutris"),
            data_dir: PathBuf::from(&home).join(".local/share/lutris"),
            cache_dir: PathBuf::from(&home).join(".cache/lutris"),
        }
    }

    /// Create config for Flatpak Lutris
    pub fn flatpak() -> Self {
        let home = std::env::var("HOME").expect("HOME environment variable not set");

        Self {
            lutris_type: LutrisType::Flatpak,
            executable_path: PathBuf::from("flatpak"),
            config_dir: PathBuf::from(&home).join(".var/app/net.lutris.Lutris/config/lutris"),
            data_dir: PathBuf::from(&home).join(".var/app/net.lutris.Lutris/data/lutris"),
            cache_dir: PathBuf::from(&home).join(".var/app/net.lutris.Lutris/cache/lutris"),
        }
    }

    /// Build a Tokio Command to run Lutris CLI asynchronously
    /// Handles different Lutris types (system, flatpak, custom)
    pub fn build_tokio_command(&self) -> TokioCommand {
        match self.lutris_type {
            LutrisType::System | LutrisType::Custom => {
                TokioCommand::new(&self.executable_path)
            }
            LutrisType::Flatpak => {
                let mut cmd = TokioCommand::new("flatpak");
                cmd.args(&["run", "net.lutris.Lutris"]);
                cmd
            }
        }
    }

    /// Check if Lutris executable exists and is accessible
    pub fn is_lutris_available(&self) -> bool {
        match self.lutris_type {
            LutrisType::System => Self::is_system_lutris_available(),
            LutrisType::Flatpak => Self::is_flatpak_lutris_available(),
            LutrisType::Custom => self.executable_path.exists(),
        }
    }

    /// Get a human-readable description of the Lutris installation
    pub fn description(&self) -> String {
        match self.lutris_type {
            LutrisType::System => format!("System Lutris ({})", self.executable_path.display()),
            LutrisType::Flatpak => "Flatpak Lutris (net.lutris.Lutris)".to_string(),
            LutrisType::Custom => format!("Custom Lutris ({})", self.executable_path.display()),
        }
    }

    // Note: Path utilities have been moved to rustris_paths module
    // Use those functions instead of methods on LutrisConfig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detect() {
        let result = LutrisConfig::auto_detect();
        // Should return either Ok with valid config or Err with install instructions
        match result {
            Ok(config) => {
                assert!(config.executable_path.to_str().is_some());
            }
            Err(msg) => {
                // Should contain installation instructions
                assert!(msg.contains("Lutris is not installed"));
            }
        }
    }

    #[test]
    fn test_system_config() {
        let config = LutrisConfig::system();
        assert_eq!(config.lutris_type, LutrisType::System);
        assert!(config.config_dir.to_str().unwrap().contains(".config/lutris"));
    }

    #[test]
    fn test_description() {
        let config = LutrisConfig::system();
        let desc = config.description();
        assert!(desc.contains("System Lutris"));
    }
}