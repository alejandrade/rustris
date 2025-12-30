import { useState } from "react";
import {
  Dialog,
  AppBar,
  Toolbar,
  IconButton,
  Typography,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import SearchGamesStep from "./addingGames/SearchGamesStep";
import InstallGameStep from "./addingGames/InstallGameStep";
import ConfigureGameStep, { type GameConfig } from "./addingGames/ConfigureGameStep";
import ViewInstallerStep from "./addingGames/ViewInstallerStep";
import type { LutrisGame, LutrisInstaller } from "../types/lutris";
import lutrisService from "../services/LutrisService";

interface AddGameModalProps {
  open: boolean;
  onClose: () => void;
}

type Step = "search" | "install" | "configure" | "view_installer";

function AddGameModal({ open, onClose }: AddGameModalProps) {
  const [currentStep, setCurrentStep] = useState<Step>("search");
  const [selectedGame, setSelectedGame] = useState<LutrisGame | null>(null);
  const [droppedExeFile, setDroppedExeFile] = useState<File | null>(null);
  const [selectedInstaller, setSelectedInstaller] = useState<LutrisInstaller | null>(null);

  const handleGameSelect = (game: LutrisGame) => {
    setSelectedGame(game);
    setCurrentStep("install");
  };

  const handleInstallerSelect = (installer: LutrisInstaller) => {
    setSelectedInstaller(installer);
    setCurrentStep("view_installer");
  };

  const handleManualAdd = (file: File) => {
    console.log("Manual add file:", file.name);
    setDroppedExeFile(file);
    setCurrentStep("configure");
  };

  const handleSaveGame = (config: GameConfig) => {
    console.log("Save game config:", config);
    // TODO: Save game to database
    handleClose();
  };

  const handleBackToInstall = () => {
    setCurrentStep("install");
    setSelectedInstaller(null);
  };

  const handleBackToSearch = () => {
    setCurrentStep("search");
    setSelectedGame(null);
    setDroppedExeFile(null);
    setSelectedInstaller(null);
  };

  const handleClose = () => {
    setCurrentStep("search");
    setSelectedGame(null);
    setDroppedExeFile(null);
    setSelectedInstaller(null);
    onClose();
  };

  const getTitle = () => {
    switch (currentStep) {
      case "search":
        return "Add Game";
      case "install":
        return `Install ${selectedGame?.name || "Game"}`;
      case "view_installer":
        return `Installer: ${selectedInstaller?.version || ""}`;
      case "configure":
        return "Configure Game";
      default:
        return "Add Game";
    }
  };

  return (
    <Dialog
      fullScreen
      open={open}
      onClose={onClose}
      PaperProps={{
        sx: {
          bgcolor: "#1b2838",
        },
      }}
    >
      <AppBar
        sx={{
          position: "relative",
          bgcolor: "#16202d",
          boxShadow: "none",
          borderBottom: "1px solid #000",
        }}
      >
        <Toolbar>
          <Typography sx={{ flex: 1, color: "#fff" }} variant="h6">
            {getTitle()}
          </Typography>
          <IconButton
            edge="end"
            color="inherit"
            onClick={handleClose}
            sx={{
              color: "#8f98a0",
              "&:hover": {
                bgcolor: "#2a475e",
                color: "#fff",
              },
            }}
          >
            <CloseIcon />
          </IconButton>
        </Toolbar>
      </AppBar>

      {/* Render the appropriate step */}
      {currentStep === "search" && (
        <SearchGamesStep
          onGameSelect={handleGameSelect}
          onManualAdd={handleManualAdd}
        />
      )}

      {currentStep === "install" && selectedGame && (
        <InstallGameStep
          game={selectedGame}
          onBack={handleBackToSearch}
          onInstallerSelect={handleInstallerSelect}
        />
      )}

      {currentStep === "view_installer" && selectedInstaller && selectedGame && (
        <ViewInstallerStep
          installer={selectedInstaller}
          onBack={handleBackToInstall}
          onSave={async (installer, editedScript) => {
            try {
              console.log("Running installer:", installer.version);
              const gameSlug = await lutrisService.runInstallerFromYaml(
                editedScript,
                selectedGame.name
              );
              console.log("Installation started, game slug:", gameSlug);
              alert(
                `Lutris installer launched!\n\n` +
                `Please complete the installation in the Lutris window.\n` +
                `The installer may ask you to:\n` +
                `  • Select installer files\n` +
                `  • Choose installation options\n` +
                `  • Wait for downloads/installations to complete\n\n` +
                `The game will appear in your library when installation is finished.`
              );
              handleClose();
            } catch (error) {
              console.error("Failed to run installer:", error);
              alert(`Installation failed: ${error}`);
            }
          }}
        />
      )}

      {currentStep === "configure" && droppedExeFile && (
        <ConfigureGameStep
          exeFile={droppedExeFile}
          onBack={handleBackToSearch}
          onSave={handleSaveGame}
        />
      )}
    </Dialog>
  );
}

export default AddGameModal;