import { Box, Typography, Button } from "@mui/material";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import type { LutrisGame } from "../../types/lutris";

interface InstallGameStepProps {
  game: LutrisGame;
  onBack: () => void;
}

function InstallGameStep({ game, onBack }: InstallGameStepProps) {

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

      {/* Placeholder for future installation functionality */}
      <Box sx={{ mt: 4, textAlign: "center", py: 6 }}>
        <Typography variant="h6" sx={{ color: "#8f98a0", mb: 2 }}>
          Lutris Installation Scripts
        </Typography>
        <Typography sx={{ color: "#8f98a0" }}>
          Installation via Lutris scripts coming soon...
        </Typography>
      </Box>
    </Box>
  );
}

export default InstallGameStep;