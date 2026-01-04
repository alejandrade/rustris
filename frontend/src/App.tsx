import { useState, useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { CssBaseline, ThemeProvider, createTheme, Box, CircularProgress, Paper, Typography, Button } from "@mui/material";
import Layout from "./components/Layout";
import Library from "./pages/Library";
import InitialSetup from "./pages/InitialSetup";
import { GameProvider } from "./context/GameContext";
import { invoke } from "@tauri-apps/api/core";

interface LutrisAvailability {
  is_available: boolean;
  installation_type: string | null;
  install_instructions: string | null;
}

const darkTheme = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: "#66c0f4",
    },
    background: {
      default: "#1b2838",
      paper: "#16202d",
    },
  },
});

function App() {
  const [lutrisAvailability, setLutrisAvailability] = useState<LutrisAvailability | null>(null);
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);
  const [checkingSetup, setCheckingSetup] = useState(true);

  useEffect(() => {
    checkLutrisAvailability();
  }, []);

  const checkLutrisAvailability = async () => {
    try {
      const availability = await invoke<LutrisAvailability>("check_lutris_availability");
      console.log("Lutris availability:", availability);
      setLutrisAvailability(availability);

      // Only check default wine version if Lutris is available
      if (availability.is_available) {
        await checkDefaultWineVersion();
      }
    } catch (error) {
      console.error("Failed to check Lutris availability:", error);
      setLutrisAvailability({
        is_available: false,
        installation_type: null,
        install_instructions: "Failed to check Lutris installation. Please ensure Lutris is installed.",
      });
    } finally {
      setCheckingSetup(false);
    }
  };

  const checkDefaultWineVersion = async () => {
    try {
      const defaultWine = await invoke<string | null>("get_lutris_global_default_wine_version");
      console.log("Lutris global default wine version:", defaultWine);
      setSetupComplete(!!defaultWine);
    } catch (error) {
      console.error("Failed to check default wine version:", error);
      // If check fails, assume setup is not complete
      setSetupComplete(false);
    }
  };

  const handleSetupComplete = () => {
    console.log("✅ Setup complete!");
    setSetupComplete(true);
  };

  if (checkingSetup) {
    return (
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
        <Box
          sx={{
            height: "100vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            bgcolor: "#1b2838",
          }}
        >
          <CircularProgress size={60} sx={{ color: "#5c7e10" }} />
        </Box>
      </ThemeProvider>
    );
  }

  // Show Lutris installation error if not available
  if (lutrisAvailability && !lutrisAvailability.is_available) {
    return (
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
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
              maxWidth: 600,
              p: 4,
              bgcolor: "#16202d",
              border: "1px solid #2a475e",
            }}
          >
            <Typography variant="h4" gutterBottom sx={{ color: "#ff6b6b" }}>
              Lutris Not Installed
            </Typography>
            <Typography variant="body1" sx={{ mb: 3, whiteSpace: "pre-line" }}>
              {lutrisAvailability.install_instructions}
            </Typography>
            <Button
              variant="contained"
              onClick={() => window.location.reload()}
              sx={{
                bgcolor: "#66c0f4",
                "&:hover": { bgcolor: "#5ab0e0" },
              }}
            >
              Check Again
            </Button>
          </Paper>
        </Box>
      </ThemeProvider>
    );
  }

  if (!setupComplete) {
    return (
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
        <GameProvider>
          <InitialSetup onComplete={handleSetupComplete} />
        </GameProvider>
      </ThemeProvider>
    );
  }

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <GameProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route index element={<Navigate to="/library" replace />} />
              <Route path="library" element={<Library />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </GameProvider>
    </ThemeProvider>
  );
}

export default App;
