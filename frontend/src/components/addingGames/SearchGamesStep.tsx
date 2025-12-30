import { useState, useEffect } from "react";
import {
  Box,
  TextField,
  Paper,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  CircularProgress,
  Typography,
  Button,
} from "@mui/material";
import SearchIcon from "@mui/icons-material/Search";
import UploadFileIcon from "@mui/icons-material/UploadFile";
import { open } from "@tauri-apps/plugin-dialog";
import { lutrisService } from "../../services/LutrisService";
import type { LutrisGame } from "../../types/lutris";

interface SearchGamesStepProps {
  onGameSelect: (game: LutrisGame) => void;
  onManualAdd: (file: File) => void;
}

function SearchGamesStep({ onGameSelect, onManualAdd }: SearchGamesStepProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<LutrisGame[]>([]);
  const [isSearching, setIsSearching] = useState(false);

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

  const handleBrowseFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Executable',
          extensions: ['exe']
        }]
      });

      if (selected && typeof selected === 'string') {
        console.log("Selected file:", selected);
        // Create a File object from the path
        const file = new File([], selected, { type: 'application/x-msdownload' });
        onManualAdd(file);
      }
    } catch (error) {
      console.error("Failed to open file:", error);
    }
  };

  return (
    <Box sx={{ p: 4, maxWidth: 800, mx: "auto", width: "100%" }}>
      {/* Search Section */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h6" sx={{ color: "#fff", mb: 2 }}>
          Search Lutris for Install Scripts
        </Typography>
        <Typography variant="body2" sx={{ color: "#8f98a0", mb: 2 }}>
          Search for games available on Lutris.net
        </Typography>
        <TextField
          fullWidth
          variant="outlined"
          placeholder="Search for games (minimum 3 characters)..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <SearchIcon sx={{ color: "#8f98a0", mr: 1 }} />
            ),
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
              "&.Mui-disabled": {
                "& fieldset": {
                  borderColor: "#000",
                },
              },
            },
            "& .MuiInputBase-input::placeholder": {
              color: "#8f98a0",
              opacity: 0.5,
            },
          }}
        />

        {/* Search Results */}
        {isSearching && (
          <Box sx={{ display: "flex", justifyContent: "center", mt: 3 }}>
            <CircularProgress sx={{ color: "#5c7e10" }} />
          </Box>
        )}

        {!isSearching && searchResults.length > 0 && (
          <Paper
            sx={{
              mt: 2,
              bgcolor: "#16202d",
              border: "1px solid #2a475e",
              maxHeight: 400,
              overflow: "auto",
            }}
          >
            <List>
              {searchResults.map((game) => (
                <ListItem key={game.slug} disablePadding>
                  <ListItemButton
                    onClick={() => onGameSelect(game)}
                    sx={{
                      "&:hover": {
                        bgcolor: "#2a475e",
                      },
                    }}
                  >
                    <ListItemText
                      primary={game.name}
                      secondary={game.year ? `${game.year}` : ""}
                      primaryTypographyProps={{
                        sx: { color: "#fff" },
                      }}
                      secondaryTypographyProps={{
                        sx: { color: "#8f98a0" },
                      }}
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Paper>
        )}

        {!isSearching && searchQuery.trim().length >= 3 && searchResults.length === 0 && (
          <Box sx={{ textAlign: "center", mt: 3 }}>
            <Typography sx={{ color: "#8f98a0" }}>
              No games found for "{searchQuery}"
            </Typography>
          </Box>
        )}
      </Box>

      {/* Add Manually Section */}
      <Box>
        <Typography variant="h6" sx={{ color: "#fff", mb: 2 }}>
          Or Add Manually
        </Typography>
        <Paper
          sx={{
            border: "2px dashed #2a475e",
            bgcolor: "#16202d",
            borderRadius: 2,
            p: 6,
            textAlign: "center",
            transition: "all 0.3s ease",
          }}
        >
          <UploadFileIcon
            sx={{
              fontSize: 64,
              color: "#8f98a0",
              mb: 2,
            }}
          />
          <Typography variant="h6" sx={{ color: "#fff", mb: 1 }}>
            Select .exe File
          </Typography>
          <Typography variant="body2" sx={{ color: "#8f98a0", mb: 3 }}>
            Browse for your game executable to add it to your library
          </Typography>
          <Button
            variant="contained"
            onClick={handleBrowseFile}
            sx={{
              bgcolor: "#5c7e10",
              "&:hover": {
                bgcolor: "#6d9512",
              },
            }}
          >
            Browse Files
          </Button>
        </Paper>
      </Box>
    </Box>
  );
}

export default SearchGamesStep;
