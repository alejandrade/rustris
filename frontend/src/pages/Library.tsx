import { useState } from "react";
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  Typography,
  CircularProgress,
  Button,
  IconButton,
} from "@mui/material";
import SportsEsportsIcon from "@mui/icons-material/SportsEsports";
import AddCircleOutlineIcon from "@mui/icons-material/AddCircleOutline";
import SettingsIcon from "@mui/icons-material/Settings";
import { convertFileSrc } from "@tauri-apps/api/core";
import GameDetail from "../components/GameDetail";
import AddGameModal from "../components/AddGameModal";
import SettingsModal from "../components/SettingsModal";
import { useGames } from "../context/GameContext";

function formatPlaytime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  if (hours === 0) return "Never played";
  return `${hours} hour${hours !== 1 ? "s" : ""}`;
}

function Library() {
  const { games, selectedGame, loading, setSelectedGame } = useGames();
  const [addGameModalOpen, setAddGameModalOpen] = useState(false);
  const [settingsModalOpen, setSettingsModalOpen] = useState(false);

  if (loading) {
    return (
      <Box
        sx={{
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          height: "100%",
          bgcolor: "#1b2838",
        }}
      >
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ display: "flex", height: "100%", bgcolor: "#1b2838" }}>
        {/* Left sidebar - Games list */}
        <Box
          sx={{
            width: 300,
            bgcolor: "#16202d",
            borderRight: "1px solid #000",
            display: "flex",
            flexDirection: "column",
          }}
        >
          {/* Games list - scrollable area */}
          <Box sx={{ flexGrow: 1, overflowY: "auto" }}>
            {games.length === 0 ? (
              <Box sx={{ p: 2, textAlign: "center" }}>
                <Typography sx={{ color: "#8f98a0", mb: 2 }}>
                  No games in library
                </Typography>
                <Typography variant="body2" sx={{ color: "#8f98a0" }}>
                  Add games to get started
                </Typography>
              </Box>
            ) : (
              <List sx={{ p: 0 }}>
                {games.map((game) => (
                  <ListItem key={game.slug} disablePadding>
                    <ListItemButton
                      selected={selectedGame?.slug === game.slug}
                      onClick={() => setSelectedGame(game)}
                      sx={{
                        "&.Mui-selected": {
                          bgcolor: "#1b2838",
                        },
                        "&:hover": {
                          bgcolor: "#2a475e",
                        },
                        py: 1,
                      }}
                    >
                      <Box
                        sx={{
                          width: 32,
                          height: 32,
                          mr: 1.5,
                          borderRadius: 0.5,
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          bgcolor: game.cover_url ? "transparent" : "#2a475e",
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
                              borderRadius: 0.5,
                            }}
                            onError={(e) => {
                              const parent = (e.target as HTMLElement).parentElement;
                              if (parent) {
                                parent.innerHTML = `<svg style="width:20px;height:20px;color:#8f98a0" viewBox="0 0 24 24"><path fill="currentColor" d="M21,6H3A1,1 0 0,0 2,7V17A1,1 0 0,0 3,18H21A1,1 0 0,0 22,17V7A1,1 0 0,0 21,6M20,16H4V8H20V16M6,15H8V9H6V15M13,15H15V9H13V15M9.5,15H11.5V9H9.5V15Z"/></svg>`;
                              }
                            }}
                          />
                        ) : (
                          <SportsEsportsIcon sx={{ fontSize: 20, color: "#8f98a0" }} />
                        )}
                      </Box>
                      <ListItemText
                        primary={game.name}
                        secondary={formatPlaytime(game.playtime)}
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
            )}
          </Box>

          {/* Bottom buttons */}
          <Box
            sx={{
              borderTop: "1px solid #000",
              p: 1.5,
              display: "flex",
              gap: 1,
            }}
          >
            <Button
              variant="contained"
              startIcon={<AddCircleOutlineIcon />}
              fullWidth
              onClick={() => setAddGameModalOpen(true)}
              disabled={true}
              sx={{
                bgcolor: "#5c7e10",
                "&:hover": {
                  bgcolor: "#6d9512",
                },
                textTransform: "none",
                fontSize: "14px",
              }}
            >
              Add Game
            </Button>
            <IconButton
              onClick={() => setSettingsModalOpen(true)}
              sx={{
                color: "#8f98a0",
                "&:hover": {
                  bgcolor: "#2a475e",
                  color: "#fff",
                },
              }}
            >
              <SettingsIcon />
            </IconButton>
          </Box>
        </Box>

        {/* Right side - Game details */}
        <Box sx={{ flexGrow: 1, p: 3, overflowY: "auto" }}>
          {selectedGame ? (
            <GameDetail game={selectedGame} />
          ) : (
            <Typography variant="h5" sx={{ color: "#8f98a0" }}>
              {games.length > 0 ? "Select a game from the library" : "Add games to get started"}
            </Typography>
          )}
        </Box>

        {/* Add Game Modal */}
        <AddGameModal
          open={addGameModalOpen}
          onClose={() => setAddGameModalOpen(false)}
        />

        {/* Settings Modal */}
        <SettingsModal
          open={settingsModalOpen}
          onClose={() => setSettingsModalOpen(false)}
        />
    </Box>
  );
}

export default Library;
