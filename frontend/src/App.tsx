import { useState, useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { CssBaseline, ThemeProvider, createTheme, Box, CircularProgress } from "@mui/material";
import Layout from "./components/Layout";
import Library from "./pages/Library";
import InitialSetup from "./pages/InitialSetup";
import { GameProvider } from "./context/GameContext";
import { invoke } from "@tauri-apps/api/core";

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
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);
  const [checkingSetup, setCheckingSetup] = useState(true);

  useEffect(() => {
    checkDefaultWineVersion();
  }, []);

  const checkDefaultWineVersion = async () => {
    try {
      const defaultWine = await invoke<string | null>("get_lutris_global_default_wine_version");
      console.log("🔍 Lutris global default wine version:", defaultWine);
      setSetupComplete(!!defaultWine);
    } catch (error) {
      console.error("Failed to check default wine version:", error);
      // If check fails, assume setup is not complete
      setSetupComplete(false);
    } finally {
      setCheckingSetup(false);
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

  if (!setupComplete) {
    return (
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
        <InitialSetup onComplete={handleSetupComplete} />
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
