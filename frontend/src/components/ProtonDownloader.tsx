import { useState, useEffect } from "react";
import {
  Box,
  Typography,
  Paper,
  Button,
  Radio,
  RadioGroup,
  FormControlLabel,
  FormControl,
  CircularProgress,
  LinearProgress,
  Link,
} from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { wineService, GeProtonRelease } from "../services/WineService";
import { useGames } from "../context/GameContext";

interface ProtonDownloaderProps {
  onDownloadComplete?: (installedPath: string) => void;
  onError?: (error: string) => void;
  autoSelectLatest?: boolean;
  installedVersions?: string[]; // Array of already installed version names
}

export default function ProtonDownloader({
  onDownloadComplete,
  onError,
  installedVersions = [],
}: ProtonDownloaderProps) {
  const { refreshWineVersions } = useGames();
  const [releases, setReleases] = useState<GeProtonRelease[]>([]);
  const [selectedRelease, setSelectedRelease] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadReleases();
  }, []);

  const loadReleases = async () => {
    setLoading(true);
    setError(null);
    try {
      const fetchedReleases = await wineService.fetchGeProtonReleases();
      console.log("📥 Fetched GE-Proton releases:", fetchedReleases);
      setReleases(fetchedReleases);

      // Don't auto-select anything - let user choose
      // (Previous auto-select logic removed for better UX)
    } catch (err) {
      const errorMsg = `Failed to fetch releases: ${err}`;
      console.error(errorMsg);
      setError(errorMsg);
      if (onError) onError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async () => {
    if (!selectedRelease) return;

    const release = releases.find((r) => r.tag_name === selectedRelease);
    if (!release) return;

    setDownloading(true);
    setError(null);

    try {
      console.log("⬇️ Downloading:", release.name);

      // Download and install the selected GE-Proton version
      const installedPath = await wineService.downloadGeProton(
        release.tag_name,
        release.download_url
      );

      console.log("✅ Installed to:", installedPath);

      // Refresh wine versions globally
      await refreshWineVersions();

      if (onDownloadComplete) {
        onDownloadComplete(installedPath);
      }

      setDownloading(false);
    } catch (err) {
      const errorMsg = `Failed to download: ${err}`;
      console.error(errorMsg);
      setError(errorMsg);
      if (onError) onError(errorMsg);
      setDownloading(false);
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
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

  if (error && releases.length === 0) {
    return (
      <Box sx={{ py: 2 }}>
        <Typography sx={{ color: "#ff6b6b", mb: 3 }}>
          {error}
        </Typography>
        <Button
          variant="contained"
          onClick={loadReleases}
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
      {downloading ? (
        <Box sx={{ textAlign: "center", py: 6 }}>
          <CircularProgress size={60} sx={{ color: "#5c7e10", mb: 3 }} />
          <Typography variant="h6" sx={{ color: "#fff", mb: 1 }}>
            Downloading and Installing...
          </Typography>
          <Typography variant="body2" sx={{ color: "#8f98a0", mb: 2 }}>
            This may take a few minutes depending on your connection speed
          </Typography>
          <LinearProgress
            sx={{
              mt: 2,
              bgcolor: "#2a475e",
              "& .MuiLinearProgress-bar": { bgcolor: "#5c7e10" },
            }}
          />
        </Box>
      ) : (
        <>
          {/* Header with title and download button */}
          <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
            <Typography variant="h6" sx={{ color: "#fff" }}>
              Select GE-Proton Version
            </Typography>
            {selectedRelease && (
              <Button
                variant="contained"
                onClick={handleDownload}
                startIcon={<DownloadIcon />}
                sx={{
                  bgcolor: "#5c7e10",
                  "&:hover": { bgcolor: "#7ba31b" },
                  textTransform: "none",
                }}
              >
                Download & Install
              </Button>
            )}
          </Box>

          <FormControl component="fieldset" fullWidth>
            <RadioGroup
              value={selectedRelease}
              onChange={(e) => setSelectedRelease(e.target.value)}
            >
              {releases.map((release) => {
                // Check if installed by comparing release name with each installed version
                // Strip any source suffix like " (Lutris2)" from installed version names
                const isInstalled = installedVersions.some(installed => {
                  const cleanName = installed.split(' (')[0]; // Remove source suffix
                  return cleanName === release.name || cleanName === release.tag_name;
                });
                return (
                  <Paper
                    key={release.tag_name}
                    sx={{
                      p: 2,
                      mb: 1.5,
                      bgcolor: isInstalled ? "#16202d" : "#1b2838",
                      border:
                        selectedRelease === release.tag_name
                          ? "2px solid #5c7e10"
                          : "2px solid transparent",
                      cursor: isInstalled ? "not-allowed" : "pointer",
                      transition: "all 0.2s",
                      opacity: isInstalled ? 0.6 : 1,
                      "&:hover": {
                        borderColor: isInstalled ? "transparent" : "#5c7e10",
                        bgcolor: isInstalled ? "#16202d" : "#1f2c3c",
                      },
                    }}
                    onClick={() => !isInstalled && setSelectedRelease(release.tag_name)}
                  >
                    <FormControlLabel
                      value={release.tag_name}
                      disabled={isInstalled}
                      control={
                        <Radio
                          disabled={isInstalled}
                          sx={{
                            color: "#8f98a0",
                            "&.Mui-checked": { color: "#5c7e10" },
                            "&.Mui-disabled": { color: "#606060" },
                          }}
                        />
                      }
                      label={
                        <Box sx={{ width: "100%" }}>
                          <Box
                            sx={{
                              display: "flex",
                              justifyContent: "space-between",
                              alignItems: "center",
                            }}
                          >
                            <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                              <Typography variant="body1" sx={{ color: isInstalled ? "#8f98a0" : "#fff", fontWeight: 500 }}>
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
                            <Typography variant="caption" sx={{ color: "#8f98a0" }}>
                              {release.size_mb.toFixed(1)} MB
                            </Typography>
                          </Box>
                          <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 0.5 }}>
                            <Typography variant="caption" sx={{ color: "#8f98a0" }}>
                              Released: {formatDate(release.published_at)}
                            </Typography>
                            <Link
                              href="#"
                              onClick={async (e) => {
                                e.preventDefault();
                                e.stopPropagation();

                                try {
                                  console.log("Opening release notes for:", release.tag_name);
                                  const releaseWindow = new WebviewWindow(`release-${release.tag_name}`, {
                                    url: `https://github.com/GloriousEggroll/proton-ge-custom/releases/tag/${release.tag_name}`,
                                    title: `${release.name} - Release Notes`,
                                    width: 1000,
                                    height: 800,
                                  });
                                  console.log("Window created:", releaseWindow);
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
                      }
                      sx={{ width: "100%", m: 0 }}
                    />
                  </Paper>
                );
              })}
            </RadioGroup>
          </FormControl>
        </>
      )}
    </Box>
  );
}
