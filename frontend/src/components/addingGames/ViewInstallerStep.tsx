import { useState } from "react";
import { Box, Typography, Button, TextField, Chip } from "@mui/material";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import SaveIcon from "@mui/icons-material/Save";
import type { LutrisInstaller } from "../../types/lutris";

interface ViewInstallerStepProps {
  installer: LutrisInstaller;
  onBack: () => void;
  onSave: (installer: LutrisInstaller, editedScript: string) => void;
}

function ViewInstallerStep({ installer, onBack, onSave }: ViewInstallerStepProps) {
  const [scriptContent, setScriptContent] = useState(
    installer.content || JSON.stringify(installer.script, null, 2)
  );

  const handleSave = () => {
    onSave(installer, scriptContent);
  };

  return (
    <Box sx={{ p: 4, maxWidth: 1200, mx: "auto", width: "100%" }}>
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
        Back to Installers
      </Button>

      {/* Installer Header */}
      <Box sx={{ mb: 3 }}>
        <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 2 }}>
          <Typography variant="h4" sx={{ color: "#fff" }}>
            {installer.version}
          </Typography>
          <Chip
            label={installer.runner}
            size="small"
            sx={{
              bgcolor: "#2a475e",
              color: "#fff",
            }}
          />
          {installer.published && (
            <Chip
              label="Published"
              size="small"
              sx={{
                bgcolor: "#5c7e10",
                color: "#fff",
              }}
            />
          )}
        </Box>
        {installer.description && (
          <Typography variant="body1" sx={{ color: "#8f98a0", mb: 1 }}>
            {installer.description}
          </Typography>
        )}
        {installer.notes && (
          <Typography variant="body2" sx={{ color: "#8f98a0" }}>
            <strong>Notes:</strong> {installer.notes}
          </Typography>
        )}
      </Box>

      {/* Script Editor */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" sx={{ color: "#fff", mb: 2 }}>
          Installation Script
        </Typography>
        <TextField
          fullWidth
          multiline
          rows={25}
          value={scriptContent}
          onChange={(e) => setScriptContent(e.target.value)}
          sx={{
            "& .MuiOutlinedInput-root": {
              fontFamily: "monospace",
              fontSize: "13px",
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
      </Box>

      {/* Action Buttons */}
      <Box sx={{ display: "flex", gap: 2 }}>
        <Button
          variant="contained"
          startIcon={<SaveIcon />}
          onClick={handleSave}
          sx={{
            bgcolor: "#5c7e10",
            "&:hover": {
              bgcolor: "#6d9512",
            },
          }}
        >
          Use This Script
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

export default ViewInstallerStep;
