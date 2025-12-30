import { useState, useEffect } from "react";
import {
  Box,
  Typography,
  Button,
  TextField,
  MenuItem,
  Paper,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  CircularProgress,
} from "@mui/material";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import InstallDesktopIcon from "@mui/icons-material/InstallDesktop";
import SearchIcon from "@mui/icons-material/Search";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { lutrisService } from "../../services/LutrisService";
import type { LutrisGame } from "../../types/lutris";

interface ConfigureGameStepProps {
  exeFile: File;
  onBack: () => void;
  onSave: (config: GameConfig) => void;
}

export interface GameConfig {
  name: string;
  exePath: string;
  coverUrl?: string;
  windowsVersion: string;
  lutrisGame?: LutrisGame;
}

const WINDOWS_VERSIONS = [
  { value: "win10", label: "Windows 10 64-bit" },
  { value: "win81", label: "Windows 8.1 64-bit" },
  { value: "win8", label: "Windows 8 64-bit" },
  { value: "win7", label: "Windows 7 64-bit" },
  { value: "vista", label: "Windows Vista" },
  { value: "win2003", label: "Windows 2003" },
  { value: "winxp", label: "Windows XP" },
  { value: "win2k", label: "Windows 2000" },
  { value: "winme", label: "Windows ME" },
  { value: "win98", label: "Windows 98" },
  { value: "win95", label: "Windows 95" },
];

function ConfigureGameStep({ exeFile, onBack, onSave }: ConfigureGameStepProps) {
  const [gameName, setGameName] = useState("");
  const [windowsVersion, setWindowsVersion] = useState("win10");
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<LutrisGame[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedLutrisGame, setSelectedLutrisGame] = useState<LutrisGame | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);

  // Don't auto-populate game name - user must enter it

  // Debounced search effect
  useEffect(() => {
    const trimmedQuery = searchQuery.trim();

    if (trimmedQuery.length < 3) {
      setSearchResults([]);
      return;
    }

    const timer = setTimeout(async () => {
      setIsSearching(true);
      try {
        const response = await lutrisService.searchGames({
          search: trimmedQuery,
        });
        setSearchResults(response.results);
      } catch (error) {
        console.error("Failed to search games:", error);
        setSearchResults([]);
      } finally {
        setIsSearching(false);
      }
    }, 800);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  const handleSelectLutrisGame = (game: LutrisGame) => {
    setSelectedLutrisGame(game);
    setGameName(game.name);
    setSearchQuery("");
    setSearchResults([]);
  };

  const handleSave = async () => {
    setIsInstalling(true);
    try {
      // Step 1: Run the installer with Wine
      console.log("Running installer:", exeFile.name);
      const programFilesPath = await invoke<string>("run_wine_installer", {
        exePath: exeFile.name,
        windowsVersion: windowsVersion,
      });

      console.log("Installer finished, program files at:", programFilesPath);
      console.log("Prompting for game exe...");

      // Step 2: Prompt user to select the actual game .exe
      const gameExePath = await open({
        multiple: false,
        title: "Select Game Executable",
        defaultPath: programFilesPath,
        filters: [{
          name: 'Executable',
          extensions: ['exe']
        }]
      });

      if (!gameExePath || typeof gameExePath !== 'string') {
        console.log("User cancelled game exe selection");
        setIsInstalling(false);
        return;
      }

      console.log("Selected game exe:", gameExePath);

      // Step 3: Save the game configuration
      const config: GameConfig = {
        name: gameName,
        exePath: gameExePath, // This is the actual game exe
        coverUrl: selectedLutrisGame?.coverart,
        windowsVersion,
        lutrisGame: selectedLutrisGame || undefined,
      };
      onSave(config);
    } catch (error) {
      console.error("Installation failed:", error);
      alert(`Installation failed: ${error}`);
      setIsInstalling(false);
    }
  };

  return (
    <Box sx={{ p: 4, maxWidth: 800, mx: "auto", width: "100%" }}>
      {/* Back Button */}
      <Button
        startIcon={<ArrowBackIcon />}
        onClick={onBack}
        sx={{
          color: "#8f98a0",
          mb: 3,
          "&:hover": {
            bgcolor: "#2a475e",
            color: "#fff",
          },
        }}
      >
        Back
      </Button>

      <Typography variant="h4" sx={{ color: "#fff", mb: 3 }}>
        Configure Game
      </Typography>

      {/* Exe File Info */}
      <Box sx={{ mb: 3, p: 2, bgcolor: "#16202d", borderRadius: 1 }}>
        <Typography variant="body2" sx={{ color: "#8f98a0" }}>
          Executable: {exeFile.name}
        </Typography>
      </Box>

      {/* Game Name with Type-ahead */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="subtitle1" sx={{ color: "#fff", mb: 1 }}>
          Game Name
        </Typography>
        <TextField
          fullWidth
          variant="outlined"
          placeholder="Search for game name..."
          value={searchQuery || gameName}
          onChange={(e) => {
            setSearchQuery(e.target.value);
            setGameName(e.target.value);
          }}
          InputProps={{
            startAdornment: <SearchIcon sx={{ color: "#8f98a0", mr: 1 }} />,
          }}
          sx={{
            "& .MuiOutlinedInput-root": {
              color: "#fff",
              bgcolor: "#16202d",
              "& fieldset": {
                borderColor: "#000",
              },
              "&:hover fieldset": {
                borderColor: "#2a475e",
              },
              "&.Mui-focused fieldset": {
                borderColor: "#5c7e10",
              },
            },
          }}
        />

        {/* Search Results */}
        {isSearching && (
          <Box sx={{ display: "flex", justifyContent: "center", mt: 2 }}>
            <CircularProgress size={30} sx={{ color: "#5c7e10" }} />
          </Box>
        )}

        {!isSearching && searchResults.length > 0 && (
          <Paper
            sx={{
              mt: 2,
              bgcolor: "#16202d",
              maxHeight: 300,
              overflowY: "auto",
            }}
          >
            <List sx={{ p: 0 }}>
              {searchResults.map((game) => (
                <ListItem key={game.id} disablePadding>
                  <ListItemButton
                    onClick={() => handleSelectLutrisGame(game)}
                    sx={{
                      "&:hover": {
                        bgcolor: "#2a475e",
                      },
                      borderBottom: "1px solid #000",
                    }}
                  >
                    {game.coverart && (
                      <Box
                        component="img"
                        src={game.coverart}
                        alt={game.name}
                        sx={{
                          width: 40,
                          height: 40,
                          objectFit: "cover",
                          borderRadius: 1,
                          mr: 2,
                        }}
                      />
                    )}
                    <ListItemText
                      primary={game.name}
                      secondary={game.year}
                      primaryTypographyProps={{
                        sx: { color: "#fff", fontSize: "14px" },
                      }}
                      secondaryTypographyProps={{
                        sx: { color: "#8f98a0", fontSize: "12px" },
                      }}
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Paper>
        )}
      </Box>

      {/* Selected Game Preview */}
      {selectedLutrisGame && (
        <Box sx={{ mb: 3, p: 2, bgcolor: "#16202d", borderRadius: 1 }}>
          <Box sx={{ display: "flex", gap: 2 }}>
            {selectedLutrisGame.coverart && (
              <Box
                component="img"
                src={selectedLutrisGame.coverart}
                alt={selectedLutrisGame.name}
                sx={{
                  width: 60,
                  height: 80,
                  objectFit: "cover",
                  borderRadius: 1,
                }}
              />
            )}
            <Box>
              <Typography variant="subtitle1" sx={{ color: "#fff" }}>
                {selectedLutrisGame.name}
              </Typography>
              <Typography variant="body2" sx={{ color: "#8f98a0" }}>
                {selectedLutrisGame.year}
              </Typography>
            </Box>
          </Box>
        </Box>
      )}

      {/* Windows Version */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="subtitle1" sx={{ color: "#fff", mb: 1 }}>
          Windows Version
        </Typography>
        <TextField
          select
          fullWidth
          value={windowsVersion}
          onChange={(e) => setWindowsVersion(e.target.value)}
          sx={{
            "& .MuiOutlinedInput-root": {
              color: "#fff",
              bgcolor: "#16202d",
              "& fieldset": {
                borderColor: "#000",
              },
              "&:hover fieldset": {
                borderColor: "#2a475e",
              },
              "&.Mui-focused fieldset": {
                borderColor: "#5c7e10",
              },
            },
          }}
        >
          {WINDOWS_VERSIONS.map((version) => (
            <MenuItem key={version.value} value={version.value}>
              {version.label}
            </MenuItem>
          ))}
        </TextField>
      </Box>

      {/* Action Buttons */}
      <Box sx={{ display: "flex", gap: 2 }}>
        <Button
          variant="contained"
          startIcon={isInstalling ? <CircularProgress size={20} sx={{ color: "#fff" }} /> : <InstallDesktopIcon />}
          onClick={handleSave}
          disabled={!gameName.trim() || isInstalling}
          sx={{
            bgcolor: "#5c7e10",
            "&:hover": {
              bgcolor: "#6d9512",
            },
            "&:disabled": {
              bgcolor: "#2a475e",
              color: "#8f98a0",
            },
          }}
        >
          {isInstalling ? "Installing..." : "Install Game"}
        </Button>
        <Button
          variant="outlined"
          onClick={onBack}
          sx={{
            color: "#8f98a0",
            borderColor: "#2a475e",
            "&:hover": {
              borderColor: "#5c7e10",
              bgcolor: "#2a475e",
              color: "#fff",
            },
          }}
        >
          Cancel
        </Button>
      </Box>
    </Box>
  );
}

export default ConfigureGameStep;
