import { useState, useEffect } from "react";
import {
  Box,
  Typography,
  Paper,
  Button,
  Divider,
  Alert,
  CircularProgress,
  LinearProgress,
  Link,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  DialogContentText,
} from "@mui/material";
import DeleteIcon from "@mui/icons-material/Delete";
import DownloadIcon from "@mui/icons-material/Download";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import { wineService, WineVersionInfo, GeProtonRelease } from "../services/WineService";
import { useGames } from "../context/GameContext";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";

interface WineRunnerManagerProps {
  onRunnersChanged?: () => void;
}

interface DownloadProgress {
  progress: number;
  downloaded: number;
  total: number;
  extracting?: boolean;
}

export default function WineRunnerManager({ onRunnersChanged }: WineRunnerManagerProps) {
  const { refreshWineVersions } = useGames();
  const [releases, setReleases] = useState<GeProtonRelease[]>([]);
  const [allInstalledVersions, setAllInstalledVersions] = useState<WineVersionInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [downloadingTag, setDownloadingTag] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [successMessage, setSuccessMessage] = useState<string>("");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<{ release: GeProtonRelease; path: string } | null>(null);

  useEffect(() => {
    loadData();

    // Listen for download progress events
    const unlisten = listen<any>("download-progress", (event) => {
      const { tag_name, downloaded, total, progress, extracting } = event.payload;

      // Only update if it's for the currently downloading tag
      if (tag_name === downloadingTag) {
        setDownloadProgress({
          progress,
          downloaded,
          total,
          extracting,
        });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [downloadingTag]);

  const loadData = async () => {
    setLoading(true);
    setErrorMessage("");
    try {
      // Load GE-Proton releases from GitHub
      const fetchedReleases = await wineService.fetchGeProtonReleases();
      console.log("📥 Fetched GE-Proton releases:", fetchedReleases);
      setReleases(fetchedReleases);

      // Load all installed versions
      const allVersions = await wineService.getAvailableVersions();
      setAllInstalledVersions(allVersions);
    } catch (error) {
      console.error("Failed to load data:", error);
      showError(`Failed to load data: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async (release: GeProtonRelease) => {
    setDownloadingTag(release.tag_name);
    setDownloadProgress(null);
    setErrorMessage("");

    try {
      console.log("⬇️ Downloading:", release.name);

      const installedPath = await wineService.downloadGeProton(
        release.tag_name,
        release.download_url
      );

      console.log("✅ Installed to:", installedPath);
      showSuccess(`${release.name} installed successfully!`);

      await loadData();
      await refreshWineVersions();
      if (onRunnersChanged) onRunnersChanged();
    } catch (error) {
      const errorMsg = `Failed to download: ${error}`;
      console.error(errorMsg);
      showError(errorMsg);
    } finally {
      setDownloadingTag(null);
      setDownloadProgress(null);
    }
  };

  const handleDelete = (release: GeProtonRelease, installedPath: string) => {
    setPendingDelete({ release, path: installedPath });
    setDeleteConfirmOpen(true);
  };

  const confirmDelete = async () => {
    if (!pendingDelete) return;

    const { release, path } = pendingDelete;
    const releaseName = allInstalledVersions.find(v =>
      v.path === path ||
      v.display_name.includes(release.name)
    )?.display_name || release.name;

    setDeleteConfirmOpen(false);
    setPendingDelete(null);

    try {
      await wineService.deleteProtonVersion(path);
      showSuccess(`Deleted ${releaseName}`);
      await loadData();
      await refreshWineVersions();
      if (onRunnersChanged) onRunnersChanged();
    } catch (error) {
      console.error("Failed to delete wine version:", error);
      showError(`Failed to delete: ${error}`);
    }
  };

  const cancelDelete = () => {
    setDeleteConfirmOpen(false);
    setPendingDelete(null);
  };

  const showSuccess = (message: string) => {
    setSuccessMessage(message);
    setTimeout(() => setSuccessMessage(""), 3000);
  };

  const showError = (message: string) => {
    setErrorMessage(message);
    setTimeout(() => setErrorMessage(""), 5000);
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };

  const getInstalledPath = (release: GeProtonRelease): string | null => {
    // Check if this release is installed
    for (const installed of allInstalledVersions) {
      const cleanName = installed.display_name.split(' (')[0];
      if (cleanName === release.name || cleanName === release.tag_name || installed.path.endsWith(release.tag_name)) {
        return installed.path;
      }
    }
    return null;
  };

  const canDelete = (path: string): boolean => {
    // Only allow deletion from Lutris wine/proton directories (not Steam)
    return path.includes('rustris');
  };

  if (loading) {
    return (
      <Box sx={{ textAlign: "center", py: 4 }}>
        <CircularProgress size={60} sx={{ color: "#5c7e10", mb: 3 }} />
        <Typography variant="h6" sx={{ color: "#fff" }}>
          Loading GE-Proton releases...
        </Typography>
      </Box>
    );
  }

  if (errorMessage && releases.length === 0) {
    return (
      <Box sx={{ py: 2 }}>
        <Typography sx={{ color: "#ff6b6b", mb: 3 }}>
          {errorMessage}
        </Typography>
        <Button
          variant="contained"
          onClick={loadData}
          sx={{
            bgcolor: "#5c7e10",
            "&:hover": { bgcolor: "#7ba31b" },
          }}
        >
          Retry
        </Button>
      </Box>
    );
  }

  return (
    <Box>
      {/* Success/Error Messages */}
      {successMessage && (
        <Alert
          severity="success"
          sx={{
            mb: 2,
            bgcolor: "#238636",
            color: "#fff",
            "& .MuiAlert-icon": { color: "#fff" },
          }}
        >
          {successMessage}
        </Alert>
      )}
      {errorMessage && (
        <Alert
          severity="error"
          sx={{
            mb: 2,
            bgcolor: "#da3633",
            color: "#fff",
            "& .MuiAlert-icon": { color: "#fff" },
          }}
        >
          {errorMessage}
        </Alert>
      )}

      {/* Header */}
      <Box sx={{ mb: 2 }}>
        <Typography variant="h6" sx={{ color: "#fff" }}>
          GE-Proton Versions
        </Typography>
        <Typography variant="body2" sx={{ color: "#8f98a0", mt: 0.5 }}>
          Download new versions or manage installed ones
        </Typography>
      </Box>

      <Divider sx={{ mb: 2, borderColor: "#2a475e" }} />

      {/* Unified Release List */}
      {releases.map((release) => {
        const installedPath = getInstalledPath(release);
        const isInstalled = installedPath !== null;
        const canDeleteThis = isInstalled && canDelete(installedPath);
        const isDownloading = downloadingTag === release.tag_name;

        return (
          <Paper
            key={release.tag_name}
            sx={{
              p: 2,
              mb: 1.5,
              bgcolor: isInstalled ? "#16202d" : "#1b2838",
              border: "2px solid transparent",
              transition: "all 0.2s",
              "&:hover": {
                bgcolor: isInstalled ? "#1a2533" : "#1f2c3c",
              },
            }}
          >
            <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <Box sx={{ flex: 1 }}>
                <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0.5 }}>
                  <Typography variant="body1" sx={{ color: "#fff", fontWeight: 500 }}>
                    {release.name}
                  </Typography>
                  {isInstalled && (
                    <Typography
                      variant="caption"
                      sx={{
                        bgcolor: "#238636",
                        color: "#fff",
                        px: 1,
                        py: 0.3,
                        borderRadius: 1,
                        fontWeight: 600,
                      }}
                    >
                      INSTALLED
                    </Typography>
                  )}
                </Box>
                <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
                  <Typography variant="caption" sx={{ color: "#8f98a0" }}>
                    Released: {formatDate(release.published_at)}
                  </Typography>
                  <Typography variant="caption" sx={{ color: "#8f98a0" }}>
                    {release.size_mb.toFixed(1)} MB
                  </Typography>
                  <Link
                    href="#"
                    onClick={async (e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      try {
                        new WebviewWindow(`release-${release.tag_name}`, {
                          url: `https://github.com/GloriousEggroll/proton-ge-custom/releases/tag/${release.tag_name}`,
                          title: `${release.name} - Release Notes`,
                          width: 1000,
                          height: 800,
                        });
                      } catch (err) {
                        console.error("Failed to open release notes:", err);
                      }
                    }}
                    sx={{
                      display: "flex",
                      alignItems: "center",
                      gap: 0.3,
                      color: "#58a6ff",
                      fontSize: "0.75rem",
                      textDecoration: "none",
                      "&:hover": {
                        textDecoration: "underline",
                      },
                    }}
                  >
                    Release Notes
                    <OpenInNewIcon sx={{ fontSize: "0.75rem" }} />
                  </Link>
                </Box>
              </Box>

              {/* Action Button */}
              {isDownloading ? (
                <Box sx={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 0.5, minWidth: 200 }}>
                  {downloadProgress?.extracting ? (
                    <Typography variant="body2" sx={{ color: "#5c7e10", fontWeight: 500 }}>
                      Extracting...
                    </Typography>
                  ) : downloadProgress ? (
                    <>
                      <Typography variant="body2" sx={{ color: "#5c7e10", fontWeight: 500 }}>
                        {downloadProgress.progress}%
                      </Typography>
                      <Typography variant="caption" sx={{ color: "#8f98a0" }}>
                        {formatBytes(downloadProgress.downloaded)} / {formatBytes(downloadProgress.total)}
                      </Typography>
                    </>
                  ) : (
                    <Typography variant="body2" sx={{ color: "#8f98a0" }}>
                      Starting...
                    </Typography>
                  )}
                </Box>
              ) : isInstalled && !canDeleteThis ? (
                <Typography
                  variant="caption"
                  sx={{
                    color: "#8f98a0",
                    fontStyle: "italic",
                    px: 2,
                  }}
                >
                  Installed (External)
                </Typography>
              ) : isInstalled && canDeleteThis ? (
                <Button
                  variant="outlined"
                  startIcon={<DeleteIcon />}
                  onClick={() => handleDelete(release, installedPath)}
                  sx={{
                    color: "#da3633",
                    borderColor: "#da3633",
                    "&:hover": {
                      borderColor: "#ff6b6b",
                      bgcolor: "rgba(218, 54, 51, 0.1)",
                    },
                  }}
                >
                  Delete
                </Button>
              ) : (
                <Button
                  variant="contained"
                  startIcon={<DownloadIcon />}
                  onClick={() => handleDownload(release)}
                  sx={{
                    bgcolor: "#5c7e10",
                    "&:hover": { bgcolor: "#7ba31b" },
                  }}
                >
                  Download
                </Button>
              )}
            </Box>

            {/* Download Progress */}
            {isDownloading && (
              <LinearProgress
                variant={downloadProgress?.extracting ? "indeterminate" : "determinate"}
                value={downloadProgress?.progress || 0}
                sx={{
                  mt: 2,
                  bgcolor: "#2a475e",
                  "& .MuiLinearProgress-bar": { bgcolor: "#5c7e10" },
                }}
              />
            )}
          </Paper>
        );
      })}

      {/* Delete Confirmation Dialog */}
      <Dialog
        open={deleteConfirmOpen}
        onClose={cancelDelete}
        PaperProps={{
          sx: {
            bgcolor: "#1b2838",
            color: "#fff",
          },
        }}
      >
        <DialogTitle sx={{ bgcolor: "#16202d", borderBottom: "1px solid #2a475e" }}>
          Confirm Deletion
        </DialogTitle>
        <DialogContent sx={{ pt: 3 }}>
          <DialogContentText sx={{ color: "#fff" }}>
            Are you sure you want to delete{" "}
            <strong>
              {pendingDelete
                ? allInstalledVersions.find(v =>
                    v.path === pendingDelete.path ||
                    v.display_name.includes(pendingDelete.release.name)
                  )?.display_name || pendingDelete.release.name
                : ""}
            </strong>
            ?
          </DialogContentText>
          <DialogContentText sx={{ color: "#8f98a0", mt: 2 }}>
            This action cannot be undone.
          </DialogContentText>
        </DialogContent>
        <DialogActions sx={{ borderTop: "1px solid #2a475e", p: 2 }}>
          <Button
            onClick={cancelDelete}
            sx={{
              color: "#8f98a0",
              "&:hover": {
                bgcolor: "#2a475e",
              },
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={confirmDelete}
            variant="contained"
            sx={{
              bgcolor: "#da3633",
              "&:hover": {
                bgcolor: "#ff6b6b",
              },
            }}
          >
            Delete
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
