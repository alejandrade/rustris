import { useState, useEffect, useRef } from "react";
import {
  Typography,
  Box,
  Paper,
  Button,
  Chip,
  Stack,
  CircularProgress,
} from "@mui/material";
import SaveIcon from "@mui/icons-material/Save";
import { gameService } from "../services/GameService";

interface GameLogProps {
  slug: string;
  gameName: string;
  isRunning: boolean;
}

const MAX_LOG_LINES = 1000; // Keep only the last 1000 lines

export default function GameLog({ slug, isRunning }: GameLogProps) {
  const [logLines, setLogLines] = useState<string[]>([]);
  const [isSavingLog, setIsSavingLog] = useState(false);
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Clear logs when switching games
  useEffect(() => {
    setLogLines([]);
    gameService.clearGameLog(slug);
  }, [slug]);

  // Listen to real-time logs when game is running
  useEffect(() => {
    if (!isRunning) return;

    // Listen for real-time log events
    const setupListener = async () => {
      const unlisten = await gameService.onGameLog(slug, (payload) => {
        // Append new lines and enforce max limit
        setLogLines((prevLines) => {
          const newLines = [...prevLines, ...payload.lines];
          // Keep only the last MAX_LOG_LINES
          if (newLines.length > MAX_LOG_LINES) {
            return newLines.slice(-MAX_LOG_LINES);
          }
          return newLines;
        });
      });

      return unlisten;
    };

    let unlistenFn: (() => void) | null = null;
    setupListener().then((fn) => {
      unlistenFn = fn;
    });

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [isRunning, slug]);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logLines]);

  const handleSaveLog = async () => {
    setIsSavingLog(true);
    try {
      const savedPath = await gameService.saveGameLog(slug);
      alert(`Log saved to: ${savedPath}`);
    } catch (error) {
      console.error("Failed to save log:", error);
      alert(`Failed to save log: ${error}`);
    } finally {
      setIsSavingLog(false);
    }
  };

  return (
    <Paper
      sx={{
        bgcolor: "#0d1117",
        color: "#c9d1d9",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Log Controls Header */}
      <Box
        sx={{
          p: 2,
          borderBottom: "1px solid #30363d",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
          <Typography variant="h6" sx={{ color: "#8b949e", fontFamily: "monospace" }}>
            Game Output
          </Typography>
          <Chip
            label={isRunning ? "Running" : "Stopped"}
            size="small"
            sx={{
              bgcolor: isRunning ? "#238636" : "#da3633",
              color: "#fff",
              fontFamily: "monospace",
            }}
          />
          <Typography sx={{ color: "#8b949e", fontSize: "0.875rem", fontFamily: "monospace" }}>
            {logLines.length} lines
          </Typography>
        </Box>
        <Stack direction="row" spacing={1}>
          <Button
            variant="outlined"
            size="small"
            startIcon={isSavingLog ? <CircularProgress size={16} /> : <SaveIcon />}
            onClick={handleSaveLog}
            disabled={isSavingLog || logLines.length === 0}
            sx={{
              color: "#a371f7",
              borderColor: "#a371f7",
              "&:hover": { bgcolor: "rgba(163, 113, 247, 0.1)", borderColor: "#a371f7" },
            }}
          >
            {isSavingLog ? "Saving..." : "Save Log"}
          </Button>
        </Stack>
      </Box>

      {/* Log Content */}
      <Box
        ref={logContainerRef}
        sx={{
          p: 2,
          flex: 1,
          overflowY: "auto",
          bgcolor: "#0d1117",
        }}
      >
        {logLines.length === 0 ? (
          <Typography sx={{ color: "#8b949e", fontFamily: "monospace" }}>
            {isRunning ? "Waiting for game output..." : "Launch the game to see output..."}
          </Typography>
        ) : (
          logLines.map((line, index) => (
            <Box
              key={index}
              sx={{
                mb: 0.5,
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
                color: "#c9d1d9",
                fontFamily: "monospace",
                fontSize: "0.85rem",
              }}
            >
              {line}
            </Box>
          ))
        )}
      </Box>
    </Paper>
  );
}