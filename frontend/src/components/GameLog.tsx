import { useState, useEffect, useRef } from "react";
import {
  Typography,
  Box,
  Paper,
  Button,
  Chip,
  Stack,
  CircularProgress,
  IconButton,
  Tooltip,
} from "@mui/material";
import SaveIcon from "@mui/icons-material/Save";
import FolderIcon from "@mui/icons-material/Folder";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import { gameService, utilityService, type OpenTarget } from "../services";
import InfoModal from "./InfoModal";

interface GameLogProps {
  slug: string;
  gameName: string;
  isRunning: boolean;
}

const MAX_LOG_LINES = 1000;

export default function GameLog({ slug, isRunning }: GameLogProps) {
  const [logLines, setLogLines] = useState<string[]>([]);
  const [isSavingLog, setIsSavingLog] = useState(false);
  const [isInfoModalOpen, setInfoModalOpen] = useState(false);
  const [infoModalContent, setInfoModalContent] = useState<{title: string, content: React.ReactNode}>({ title: '', content: null });
  const logContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setLogLines([]);
    gameService.clearGameLog(slug);
  }, [slug]);

  useEffect(() => {
    if (!isRunning) return;
    const setupListener = async () => {
      const unlisten = await gameService.onGameLog(slug, (payload) => {
        setLogLines((prevLines) => {
          const newLines = [...prevLines, ...payload.lines];
          if (newLines.length > MAX_LOG_LINES) {
            return newLines.slice(-MAX_LOG_LINES);
          }
          return newLines;
        });
      });
      return unlisten;
    };
    let unlistenFn: (() => void) | null = null;
    setupListener().then((fn) => { unlistenFn = fn; });
    return () => { if (unlistenFn) { unlistenFn(); } };
  }, [isRunning, slug]);

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logLines]);

  const handleOpenLogFile = async (savedLogPath: string) => {
    if (!savedLogPath) return;
    try {
      console.log("Opening log file: ", savedLogPath);
      const target: OpenTarget = { type: 'path', value: savedLogPath };
      await utilityService.open(target);
    } catch (e) {
      console.error('Failed to open log file', e);
    }
  }

  const handleOpenLogDirectory = async (savedLogPath: string) => {
    if (!savedLogPath) return;
    try {
      console.log("Opening log directory: ", savedLogPath);
      const target: OpenTarget = { type: 'directory', value: savedLogPath };
      await utilityService.open(target);
    } catch (e) {
      console.error('Failed to open log directory', e);
    }
  }

  const handleSaveLog = async () => {
    setIsSavingLog(true);
    try {
      const savedPath = await gameService.saveGameLog(slug);
      const fileName = savedPath.split('/').pop() || '';
      setInfoModalContent({
        title: "Log File Saved",
        content: (
          <>
            <Typography sx={{ mb: 1 }}>Diagnosis file saved to Downloads folder!</Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', background: '#0d1117', p: 1, borderRadius: 1, mb: 2 }}>
              <Typography variant="body2" sx={{ fontFamily: 'monospace', flexGrow: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {fileName}
              </Typography>
              <Tooltip title="Show in folder">
                <IconButton onClick={() => handleOpenLogDirectory(savedPath)} size="small">
                  <FolderIcon sx={{ color: '#a371f7' }} />
                </IconButton>
              </Tooltip>
              <Tooltip title="Open file">
                <IconButton onClick={() => handleOpenLogFile(savedPath)} size="small">
                  <OpenInNewIcon sx={{ color: '#a371f7' }} />
                </IconButton>
              </Tooltip>
            </Box>
            <Typography>If you need help, send this file to the developer:</Typography>
            <Typography
              variant="body2"
              sx={{
                fontFamily: 'monospace',
                background: '#0d1117',
                p: 1,
                borderRadius: 1,
                mt: 1,
                cursor: 'pointer',
                color: '#a371f7',
                display: 'inline-block',
                textDecoration: 'underline',
                '&:hover': {
                  color: '#c4b5fd', // Lighter purple on hover
                },
              }}
              onClick={() => {
                const target: OpenTarget = { type: 'url', value: 'https://github.com/alejandrade/rustris' };
                utilityService.open(target);
              }}
            >
              https://github.com/alejandrade/rustris
            </Typography>
          </>
        ),
      });
      setInfoModalOpen(true);
    } catch (error) {
      console.error("Failed to save log:", error);
      setInfoModalContent({
        title: "Error Saving Log",
        content: `Failed to save diagnosis file: ${error}`,
      });
      setInfoModalOpen(true);
    } finally {
      setIsSavingLog(false);
    }
  };

  return (
    <>
      <InfoModal
        open={isInfoModalOpen}
        onClose={() => setInfoModalOpen(false)}
        title={infoModalContent.title}
      >
        {infoModalContent.content}
      </InfoModal>
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
    </>
  );
}