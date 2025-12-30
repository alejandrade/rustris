import {
  Box,
  Typography,
  Paper,
} from "@mui/material";
import { invoke } from "@tauri-apps/api/core";
import ProtonDownloader from "../components/ProtonDownloader";

interface InitialSetupProps {
  onComplete: () => void;
}

export default function InitialSetup({ onComplete }: InitialSetupProps) {
  const handleDownloadComplete = async (installedPath: string) => {
    try {
      // Set as Lutris's global default
      await invoke("set_lutris_global_default_wine_version", {
        winePath: installedPath,
      });

      console.log("✅ Set as Lutris global default");

      // Complete setup
      onComplete();
    } catch (err) {
      console.error("Failed to set Lutris global default:", err);
      alert(`Failed to set global default: ${err}`);
    }
  };

  return (
    <Box
      sx={{
        height: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        bgcolor: "#1b2838",
        p: 3,
      }}
    >
      <Paper
        sx={{
          p: 4,
          bgcolor: "#16202d",
          maxWidth: 700,
          width: "100%",
        }}
      >
        <Typography variant="h4" sx={{ color: "#fff", mb: 1 }}>
          Welcome to Rustris
        </Typography>
        <Typography variant="body1" sx={{ color: "#8f98a0", mb: 4 }}>
          To get started, please select a GE-Proton version to download and install.
          This will be used as the default compatibility layer for running Windows games.
        </Typography>

        <ProtonDownloader
          onDownloadComplete={handleDownloadComplete}
          autoSelectLatest={true}
        />
      </Paper>
    </Box>
  );
}