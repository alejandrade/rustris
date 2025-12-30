import { useState, useEffect, useRef } from "react";
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

  // Search functionality disabled
  // useEffect(() => {
  //   const trimmedQuery = searchQuery.trim();
  //   if (trimmedQuery.length < 3) {
  //     setSearchResults([]);
  //     return;
  //   }
  //   const timer = setTimeout(async () => {
  //     setIsSearching(true);
  //     try {
  //       const response = await lutrisService.searchGames({
  //         search: trimmedQuery,
  //       });
  //       setSearchResults(response.results);
  //     } catch (error) {
  //       console.error("Failed to search games:", error);
  //       setSearchResults([]);
  //     } finally {
  //       setIsSearching(false);
  //     }
  //   }, 800);
  //   return () => clearTimeout(timer);
  // }, [searchQuery]);

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
          Searching Lutris for Install Scripts
        </Typography>
        <Typography variant="body2" sx={{ color: "#8f98a0", mb: 2 }}>
          Coming soon...
        </Typography>
        <TextField
          fullWidth
          variant="outlined"
          placeholder="Search disabled - coming soon"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          disabled
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
