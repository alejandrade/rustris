import { useState, useEffect, useRef } from "react";
import {
  Dialog,
  AppBar,
  Toolbar,
  IconButton,
  Typography,
  Box,
  Paper,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { listen } from "@tauri-apps/api/event";

interface GameLogProps {
  gameId: number;
  gameName: string;
  open: boolean;
  onClose: () => void;
}

interface LogEntry {
  stream: "stdout" | "stderr";
  message: string;
  timestamp: Date;
}

export default function GameLog({ gameId, gameName, open, onClose }: GameLogProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;

    // Clear logs when dialog opens
    setLogs([]);

    // Listen for game log events
    const unlisten = listen<[number, string, string]>("game-log", (event) => {
      const [logGameId, stream, message] = event.payload;

      // Only show logs for this game
      if (logGameId === gameId) {
        setLogs((prevLogs) => [
          ...prevLogs,
          {
            stream: stream as "stdout" | "stderr",
            message,
            timestamp: new Date(),
          },
        ]);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [open, gameId]);

  useEffect(() => {
    // Auto-scroll to bottom when new logs arrive
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <Dialog open={open} onClose={onClose} fullScreen>
      <AppBar sx={{ position: "relative", bgcolor: "#16202d" }}>
        <Toolbar>
          <IconButton edge="start" color="inherit" onClick={onClose} aria-label="close">
            <CloseIcon />
          </IconButton>
          <Typography sx={{ ml: 2, flex: 1 }} variant="h6">
            Game Output - {gameName}
          </Typography>
        </Toolbar>
      </AppBar>

      <Box sx={{ bgcolor: "#0d1117", height: "100%", overflow: "hidden" }}>
        <Paper
          sx={{
            height: "100%",
            bgcolor: "#0d1117",
            color: "#c9d1d9",
            p: 2,
            overflowY: "auto",
            fontFamily: "monospace",
            fontSize: "0.9rem",
          }}
        >
          {logs.length === 0 ? (
            <Typography sx={{ color: "#8b949e", fontFamily: "monospace" }}>
              Waiting for game output...
            </Typography>
          ) : (
            logs.map((log, index) => (
              <Box
                key={index}
                sx={{
                  mb: 0.5,
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-word",
                }}
              >
                <span
                  style={{
                    color: log.stream === "stderr" ? "#ff7b72" : "#58a6ff",
                    marginRight: "8px",
                  }}
                >
                  [{log.timestamp.toLocaleTimeString()}]
                </span>
                <span
                  style={{
                    color: log.stream === "stderr" ? "#ff7b72" : "#c9d1d9",
                  }}
                >
                  {log.message}
                </span>
              </Box>
            ))
          )}
          <div ref={logsEndRef} />
        </Paper>
      </Box>
    </Dialog>
  );
}