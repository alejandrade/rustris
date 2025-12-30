import { useState, useEffect } from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  Box,
  Typography,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  IconButton,
  Divider,
  Alert,
  Button
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import { wineService } from "../services";
import WineRunnerManager from "./WineRunnerManager";

interface WineVersionInfo {
  path: string;
  display_name: string;
}

interface SettingsModalProps {
  open: boolean;
  onClose: () => void;
}

type SettingsView = "main" | "manage-runners";

export default function SettingsModal({ open, onClose }: SettingsModalProps) {
  const [currentView, setCurrentView] = useState<SettingsView>("main");
  const [wineVersions, setWineVersions] = useState<WineVersionInfo[]>([]);
  const [defaultWineVersion, setDefaultWineVersion] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string>("");

  // Load wine versions and current default
  useEffect(() => {
    if (open) {
      loadSettings();
      setCurrentView("main"); // Reset to main view when modal opens
    }
  }, [open]);

  const loadSettings = async () => {
    setLoading(true);
    try {
      // Load available wine versions
      const versions = await wineService.getAvailableVersions();
      setWineVersions(versions);

      // Load current default
      const lutrisDefault = await wineService.getGlobalDefaultVersion();
      setDefaultWineVersion(lutrisDefault || "");
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleDefaultWineChange = async (newVersion: string) => {
    try {
      await wineService.setGlobalDefaultVersion(newVersion);
      setDefaultWineVersion(newVersion);
      setSuccessMessage("Default Wine/Proton version updated successfully!");
      setTimeout(() => setSuccessMessage(""), 3000);
      console.log(`Updated Lutris default wine version to: ${newVersion}`);
    } catch (error) {
      console.error("Failed to update default wine version:", error);
      alert(`Failed to update default wine version: ${error}`);
    }
  };

  const handleRunnersChanged = async () => {
    // Reload wine versions when runners are added/deleted
    await loadSettings();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: {
          bgcolor: "#1b2838",
          color: "#fff",
          minHeight: "60vh",
        },
      }}
    >
      <DialogTitle
        sx={{
          bgcolor: "#16202d",
          borderBottom: "1px solid #000",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          py: 2,
        }}
      >
        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
          {currentView !== "main" && (
            <IconButton
              onClick={() => setCurrentView("main")}
              sx={{
                color: "#8f98a0",
                "&:hover": {
                  color: "#fff",
                  bgcolor: "#2a475e",
                },
              }}
            >
              <ArrowBackIcon />
            </IconButton>
          )}
          <Typography variant="h5" sx={{ fontWeight: 600 }}>
            {currentView === "main" ? "Settings" : "Manage Wine Runners"}
          </Typography>
        </Box>
        <IconButton
          onClick={onClose}
          sx={{
            color: "#8f98a0",
            "&:hover": {
              color: "#fff",
              bgcolor: "#2a475e",
            },
          }}
        >
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent sx={{ p: 4 }}>
        {/* Success Message */}
        {successMessage && (
          <Alert
            severity="success"
            sx={{
              mb: 3,
              bgcolor: "#238636",
              color: "#fff",
              "& .MuiAlert-icon": {
                color: "#fff",
              },
            }}
          >
            {successMessage}
          </Alert>
        )}

        {/* Main Settings View */}
        {currentView === "main" && (
          <>
            {/* Wine/Proton Settings Section */}
            <Box sx={{ mb: 4 }}>
              <Typography variant="h6" sx={{ mb: 2, fontWeight: 500 }}>
                Wine/Proton Configuration
              </Typography>
              <Divider sx={{ mb: 3, borderColor: "#2a475e" }} />

              <Box sx={{ mb: 3 }}>
                <Typography variant="body2" sx={{ color: "#8f98a0", mb: 2 }}>
                  Set the default Wine/Proton version for all games in Lutris. Individual games can
                  override this setting.
                </Typography>

                <FormControl
                  fullWidth
                  disabled={loading}
                  sx={{
                    "& .MuiOutlinedInput-root": {
                      bgcolor: "#2a475e",
                      color: "#fff",
                      "& fieldset": {
                        borderColor: "#3a5a6e",
                      },
                      "&:hover fieldset": {
                        borderColor: "#5c7e10",
                      },
                      "&.Mui-focused fieldset": {
                        borderColor: "#5c7e10",
                      },
                      "&.Mui-disabled": {
                        bgcolor: "#1b2838",
                        color: "#8f98a0",
                      },
                    },
                    "& .MuiInputLabel-root": {
                      color: "#8f98a0",
                      "&.Mui-focused": {
                        color: "#5c7e10",
                      },
                      "&.Mui-disabled": {
                        color: "#606060",
                      },
                    },
                  }}
                >
                  <InputLabel id="default-wine-label">Default Wine/Proton Version</InputLabel>
                  <Select
                    labelId="default-wine-label"
                    value={defaultWineVersion}
                    label="Default Wine/Proton Version"
                    onChange={(e) => handleDefaultWineChange(e.target.value)}
                    displayEmpty
                    MenuProps={{
                      PaperProps: {
                        sx: {
                          bgcolor: "#1b2838",
                          color: "#fff",
                          "& .MuiMenuItem-root": {
                            "&:hover": {
                              bgcolor: "#2a475e",
                            },
                            "&.Mui-selected": {
                              bgcolor: "#2a475e",
                              "&:hover": {
                                bgcolor: "#3a5a6e",
                              },
                            },
                          },
                        },
                      },
                    }}
                  >
                    {!defaultWineVersion && (
                      <MenuItem value="" disabled>
                        <em>No default version configured</em>
                      </MenuItem>
                    )}
                    {defaultWineVersion && !wineVersions.find(v => v.path === defaultWineVersion) && (
                      <MenuItem value={defaultWineVersion}>
                        {defaultWineVersion.split('/').pop() || defaultWineVersion} (current)
                      </MenuItem>
                    )}
                    {wineVersions.map((version) => (
                      <MenuItem key={version.path} value={version.path}>
                        {version.display_name}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>

                {/* Manage Runners Button */}
                <Button
                  fullWidth
                  variant="outlined"
                  onClick={() => setCurrentView("manage-runners")}
                  sx={{
                    mt: 2,
                    color: "#5c7e10",
                    borderColor: "#5c7e10",
                    "&:hover": {
                      borderColor: "#7ba31b",
                      bgcolor: "rgba(92, 126, 16, 0.1)",
                    },
                  }}
                >
                  Manage Wine Runners
                </Button>
              </Box>
            </Box>

            {/* Additional settings sections can be added here */}
            <Box sx={{ mb: 4 }}>
              <Typography variant="h6" sx={{ mb: 2, fontWeight: 500 }}>
                About
              </Typography>
              <Divider sx={{ mb: 3, borderColor: "#2a475e" }} />
              <Typography variant="body2" sx={{ color: "#8f98a0" }}>
                Lutris2 - A modern interface for Lutris
              </Typography>
              <Typography variant="body2" sx={{ color: "#8f98a0", mt: 1 }}>
                Version: 0.1.0
              </Typography>
            </Box>
          </>
        )}

        {/* Wine Runner Management View */}
        {currentView === "manage-runners" && (
          <WineRunnerManager onRunnersChanged={handleRunnersChanged} />
        )}
      </DialogContent>
    </Dialog>
  );
}
