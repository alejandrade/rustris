import { useState, useEffect } from "react";
import { Box, Typography, Button, CircularProgress, Card, CardContent, Chip } from "@mui/material";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import type { LutrisGame, LutrisInstaller } from "../../types/lutris";
import lutrisService from "../../services/LutrisService";

interface InstallGameStepProps {
  game: LutrisGame;
  onBack: () => void;
  onInstallerSelect: (installer: LutrisInstaller) => void;
}

function InstallGameStep({ game, onBack, onInstallerSelect }: InstallGameStepProps) {
  const [installers, setInstallers] = useState<LutrisInstaller[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadInstallers();
  }, [game.slug]);

  const loadInstallers = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await lutrisService.getInstallers(game.slug);
      setInstallers(result);
    } catch (err) {
      console.error("Failed to load installers:", err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box sx={{ p: 4, maxWidth: 1000, mx: "auto", width: "100%" }}>
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
        Back to Search
      </Button>

      {/* Game Info */}
      <Box sx={{ display: "flex", gap: 3, mb: 4 }}>
        {game.coverart && (
          <Box
            component="img"
            src={game.coverart}
            alt={game.name}
            sx={{
              width: 150,
              height: 200,
              objectFit: "cover",
              borderRadius: 2,
            }}
          />
        )}
        <Box sx={{ flex: 1 }}>
          <Typography variant="h4" sx={{ color: "#fff", mb: 1 }}>
            {game.name}
          </Typography>
          {game.year && (
            <Typography variant="body1" sx={{ color: "#8f98a0", mb: 1 }}>
              {game.year}
            </Typography>
          )}
          {game.platforms && game.platforms.length > 0 && (
            <Typography variant="body2" sx={{ color: "#8f98a0", mb: 2 }}>
              Platforms: {game.platforms.map((p: any) => p.name || p).join(", ")}
            </Typography>
          )}
          {game.description && (
            <Typography variant="body2" sx={{ color: "#8f98a0" }}>
              {game.description}
            </Typography>
          )}
        </Box>
      </Box>

      {/* Installers */}
      <Box>
        <Typography variant="h6" sx={{ color: "#fff", mb: 2 }}>
          Available Installation Scripts
        </Typography>

        {loading && (
          <Box sx={{ display: "flex", justifyContent: "center", py: 6 }}>
            <CircularProgress sx={{ color: "#5c7e10" }} />
          </Box>
        )}

        {error && (
          <Box sx={{ textAlign: "center", py: 6 }}>
            <Typography sx={{ color: "#ff6b6b", mb: 2 }}>
              Failed to load installers: {error}
            </Typography>
            <Button
              onClick={loadInstallers}
              sx={{
                bgcolor: "#5c7e10",
                color: "#fff",
                "&:hover": { bgcolor: "#6d9512" },
              }}
            >
              Retry
            </Button>
          </Box>
        )}

        {!loading && !error && installers.length === 0 && (
          <Box sx={{ textAlign: "center", py: 6 }}>
            <Typography sx={{ color: "#8f98a0" }}>
              No installers found for this game.
            </Typography>
          </Box>
        )}

        {!loading && !error && installers.length > 0 && (
          <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
            {installers.map((installer, index) => (
              <Card
                key={index}
                sx={{
                  bgcolor: "#16202d",
                  border: "1px solid #2a475e",
                  cursor: "pointer",
                  "&:hover": {
                    borderColor: "#5c7e10",
                    bgcolor: "#1a2633",
                  },
                }}
                onClick={() => onInstallerSelect(installer)}
              >
                <CardContent>
                  <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
                    <Typography variant="h6" sx={{ color: "#fff" }}>
                      {installer.version}
                    </Typography>
                    <Chip
                      label={installer.runner}
                      size="small"
                      sx={{ bgcolor: "#2a475e", color: "#fff" }}
                    />
                    {installer.published && (
                      <Chip
                        label="Published"
                        size="small"
                        sx={{ bgcolor: "#5c7e10", color: "#fff" }}
                      />
                    )}
                  </Box>
                  {installer.description && (
                    <Typography variant="body2" sx={{ color: "#8f98a0" }}>
                      {installer.description}
                    </Typography>
                  )}
                </CardContent>
              </Card>
            ))}
          </Box>
        )}
      </Box>
    </Box>
  );
}

export default InstallGameStep;