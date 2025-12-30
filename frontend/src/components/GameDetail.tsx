import { useState, useEffect } from "react";
import { Box, Typography, Paper, Button, Chip, Stack, CircularProgress, Select, MenuItem, FormControl, InputLabel } from "@mui/material";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import SportsEsportsIcon from "@mui/icons-material/SportsEsports";
import DeleteIcon from "@mui/icons-material/Delete";
import VisibilityIcon from "@mui/icons-material/Visibility";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Game, useGames } from "../context/GameContext";
import { gameService, wineService } from "../services";

interface GameDetailProps {
  game: Game;
}

export default function GameDetail({ game }: GameDetailProps) {
  const { availableWineVersions } = useGames();
  const [showLogs, setShowLogs] = useState(false);
  const [isProcessRunning, setIsProcessRunning] = useState(false);
  const [logContent, setLogContent] = useState<string>("");
  const [isLoadingLogs, setIsLoadingLogs] = useState(false);
  const [isSavingLogs, setIsSavingLogs] = useState(false);
  const [selectedWineVersion, setSelectedWineVersion] = useState<string>("");

  const handlePlay = async () => {
    try {
      await gameService.launchGame(game.slug);
      setIsProcessRunning(true);
      setShowLogs(true); // Auto-show log controls when game launches
      setLogContent("");
      console.log(`Game launched: ${game.slug}`);
    } catch (error) {
      console.error("Failed to launch game:", error);
      alert(`Failed to launch game: ${error}`);
    }
  };

  const handleGetLogs = async () => {
    setIsLoadingLogs(true);
    try {
      const content = await gameService.getGameLog(game.slug);
      setLogContent(content);
      setShowLogs(true);
      console.log(`Retrieved log content (${content.length} bytes)`);
    } catch (error) {
      console.error("Failed to get logs:", error);
      alert(`Failed to get logs: ${error}`);
    } finally {
      setIsLoadingLogs(false);
    }
  };

  const handleSaveLogs = async () => {
    setIsSavingLogs(true);
    try {
      const savedPath = await gameService.saveGameLog(game.slug);
      alert(`Log saved to: ${savedPath}`);
    } catch (error) {
      console.error("Failed to save logs:", error);
      alert(`Failed to save logs: ${error}`);
    } finally {
      setIsSavingLogs(false);
    }
  };

  // Check process status periodically using slug
  useEffect(() => {
    const checkProcess = async () => {
      try {
        console.log("checking if process is running");
        const running = await gameService.checkGameRunning(game.slug);
        setIsProcessRunning(running);
      } catch (error) {
        console.error("Failed to check process status:", error);
        setIsProcessRunning(false);
      }
    };

    // Check immediately
    checkProcess();

    // Then check every 2 seconds
    const interval = setInterval(checkProcess, 2000);

    return () => clearInterval(interval);
  }, [game.slug]);

  // Set current wine version selection
  useEffect(() => {
    const setCurrentWineVersion = async () => {
      try {
        // Use game's wine version if set, otherwise use Lutris's default
        console.log(game);
        if (game.wine_version) {
          setSelectedWineVersion(game.wine_version);
        } else {
          // Get Lutris's default wine version (already resolved to full path)
          const lutrisDefaultPath = await wineService.getGlobalDefaultVersion();
          setSelectedWineVersion(lutrisDefaultPath || "");
        }
      } catch (error) {
        console.error("Failed to set wine version:", error);
      }
    };

    setCurrentWineVersion();
  }, [game.slug, game.wine_version]);

  const handleWineVersionChange = async (newVersion: string) => {
    try {
      await gameService.updateWineVersion(game.slug, newVersion);
      setSelectedWineVersion(newVersion);
      console.log(`Updated wine version to: ${newVersion}`);
    } catch (error) {
      console.error("Failed to update wine version:", error);
      alert(`Failed to update wine version: ${error}`);
    }
  };

  return (
    <Box>
      {/* Header with cover and title */}
      <Box sx={{ display: "flex", alignItems: "flex-start", mb: 3 }}>
        {/* Cover Art */}
        <Box
          sx={{
            width: 200,
            height: 280,
            mr: 3,
            borderRadius: 2,
            overflow: "hidden",
            bgcolor: "#2a475e",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
            boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
          }}
        >
          {game.cover_url ? (
            <Box
              component="img"
              src={convertFileSrc(game.cover_url)}
              alt={game.name}
              sx={{
                width: "100%",
                height: "100%",
                objectFit: "cover",
              }}
              onError={(e) => {
                const target = e.target as HTMLElement;
                target.style.display = "none";
                const parent = target.parentElement;
                if (parent) {
                  parent.innerHTML = `<svg style="width:80px;height:80px;color:#8f98a0" viewBox="0 0 24 24"><path fill="currentColor" d="M21,6H3A1,1 0 0,0 2,7V17A1,1 0 0,0 3,18H21A1,1 0 0,0 22,17V7A1,1 0 0,0 21,6M20,16H4V8H20V16M6,15H8V9H6V15M13,15H15V9H13V15M9.5,15H11.5V9H9.5V15Z"/></svg>`;
                }
              }}
            />
          ) : (
            <SportsEsportsIcon sx={{ fontSize: 80, color: "#8f98a0" }} />
          )}
        </Box>

        {/* Title and metadata */}
        <Box sx={{ flex: 1, minWidth: 0 }}>
          <Typography
            variant="h3"
            sx={{
              color: "#fff",
              fontWeight: 600,
              mb: 2,
              lineHeight: 1.2,
            }}
          >
            {game.name}
          </Typography>

          <Box sx={{ display: "flex", gap: 1, mb: 2, flexWrap: "wrap" }}>
            <Chip
              label={game.runner || "Unknown"}
              size="small"
              sx={{
                bgcolor: "#2a475e",
                color: "#fff",
                fontWeight: 500,
              }}
            />
          </Box>

          {/* Wine Version Selector */}
          <Box sx={{ mb: 2 }}>
            <FormControl
              fullWidth
              size="small"
              sx={{
                maxWidth: 400,
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
                },
                "& .MuiInputLabel-root": {
                  color: "#8f98a0",
                  "&.Mui-focused": {
                    color: "#5c7e10",
                  },
                },
              }}
            >
              <InputLabel id="wine-version-label">Wine/Proton Version</InputLabel>
              <Select
                labelId="wine-version-label"
                id="wine-version-select"
                value={selectedWineVersion}
                label="Wine/Proton Version"
                onChange={(e) => handleWineVersionChange(e.target.value)}
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
                {!selectedWineVersion && (
                  <MenuItem value="" disabled>
                    <em>No Wine version configured</em>
                  </MenuItem>
                )}
                {selectedWineVersion && !availableWineVersions.find(v => v.path === selectedWineVersion) && (
                  <MenuItem value={selectedWineVersion}>
                    {selectedWineVersion.split('/').pop() || selectedWineVersion} (current)
                  </MenuItem>
                )}
                {availableWineVersions.map((version) => (
                  <MenuItem key={version.path} value={version.path}>
                    {version.display_name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>

          <Box sx={{ display: "flex", gap: 2, alignItems: "center" }}>
            <Button
              variant="contained"
              size="large"
              startIcon={<PlayArrowIcon />}
              onClick={handlePlay}
              sx={{
                bgcolor: "#5c7e10",
                "&:hover": { bgcolor: "#7ba31b" },
                fontSize: "1.1rem",
                px: 4,
                py: 1.5,
                fontWeight: 600,
              }}
            >
              Play
            </Button>
          </Box>
        </Box>
      </Box>

      {/* Game Logs */}
      {showLogs && (
        <Paper
          sx={{
            bgcolor: "#0d1117",
            color: "#c9d1d9",
            mb: 2,
          }}
        >
          {/* Log Controls Header */}
          <Box sx={{ p: 2, borderBottom: "1px solid #30363d", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
              <Typography
                variant="h6"
                sx={{ color: "#8b949e", fontFamily: "monospace" }}
              >
                Game Output
              </Typography>
              <Chip
                label={isProcessRunning ? "Running" : "Stopped"}
                size="small"
                sx={{
                  bgcolor: isProcessRunning ? "#238636" : "#da3633",
                  color: "#fff",
                  fontFamily: "monospace",
                }}
              />
            </Box>
            <Stack direction="row" spacing={1}>
              <Button
                variant="outlined"
                size="small"
                startIcon={isLoadingLogs ? <CircularProgress size={16} sx={{ color: "#58a6ff" }} /> : <VisibilityIcon />}
                onClick={handleGetLogs}
                disabled={isLoadingLogs}
                sx={{
                  color: "#58a6ff",
                  borderColor: "#58a6ff",
                  "&:hover": { bgcolor: "rgba(88, 166, 255, 0.1)", borderColor: "#58a6ff" },
                }}
              >
                {isLoadingLogs ? "Loading..." : "Get Logs"}
              </Button>
              <Button
                variant="outlined"
                size="small"
                startIcon={isSavingLogs ? <CircularProgress size={16} sx={{ color: "#a371f7" }} /> : <DeleteIcon />}
                onClick={handleSaveLogs}
                disabled={isSavingLogs}
                sx={{
                  color: "#a371f7",
                  borderColor: "#a371f7",
                  "&:hover": { bgcolor: "rgba(163, 113, 247, 0.1)", borderColor: "#a371f7" },
                }}
              >
                {isSavingLogs ? "Saving..." : "Save Logs"}
              </Button>
            </Stack>
          </Box>

          {/* Log Content */}
          <Box sx={{ p: 3, height: "400px", overflowY: "auto" }}>
          {logContent ? (
            <Box
              sx={{
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
                fontFamily: "monospace",
                fontSize: "0.85rem",
                color: "#c9d1d9",
              }}
            >
              {logContent}
            </Box>
          ) : (
            <Typography sx={{ color: "#8b949e", fontFamily: "monospace" }}>
              Click "Get Logs" to view game output...
            </Typography>
          )}
          </Box>
        </Paper>
      )}
    </Box>
  );
}