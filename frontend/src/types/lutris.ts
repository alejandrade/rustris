/**
 * TypeScript types matching the Lutris2 backend API
 * See API.md for full documentation
 */

// ===== Game Data =====

/**
 * Game data from Lutris database and config files
 * Returned by get_games() command
 */
export interface GameData {
  slug: string;                    // Unique game identifier
  name: string;                    // Display name
  runner: string | null;           // Runner type (e.g., "wine", "linux")
  directory: string | null;        // Game installation directory
  playtime: number;                // Total playtime in seconds
  last_played: string | null;      // RFC3339 timestamp
  executable: string | null;       // Full path to game executable
  wine_version: string | null;     // Wine/Proton version path
  wine_prefix: string | null;      // Wine prefix path
  environment_vars: string | null; // Env vars (KEY=VALUE;KEY2=VALUE2)
  cover_url: string | null;        // Path to cover image
  debug_output: boolean;           // Debug logging enabled
}

// ===== Lutris.net API Types =====

/**
 * Game search result from Lutris.net
 */
export interface LutrisGame {
  id: number;
  slug: string;
  name: string;
  year: number | null;
  coverart: string;
  icon: string;
  banner: string;
  description: string;
  website: string;
  platforms: string[];
  genres: string[];
  publisher: string;
  developer: string;
  installers?: LutrisInstaller[];
}

/**
 * Installer script from Lutris.net
 */
export interface LutrisInstaller {
  id: number;
  slug: string;
  version: string;
  description: string;
  notes: string;
  credits: string;
  created_at: string;
  updated_at: string;
  draft: boolean;
  published: boolean;
  rating: string;
  user: string;
  runner: string;
  script: InstallerScript;
  content: string;
}

export interface InstallerScript {
  files?: InstallerFile[];
  game?: {
    exe?: string;
    prefix?: string;
    args?: string;
    [key: string]: any;
  };
  installer?: InstallerStep[];
  system?: {
    env?: Record<string, string>;
    [key: string]: any;
  };
  wine?: {
    version?: string;
    [key: string]: any;
  };
}

export interface InstallerFile {
  [key: string]: string | { url: string; filename?: string };
}

export interface InstallerStep {
  task?: string;
  [key: string]: any;
}

export interface LutrisGameListResponse {
  count: number;
  next: string | null;
  previous: string | null;
  results: LutrisGame[];
}

export interface LutrisSearchParams {
  search?: string;
  platforms?: string;
  genres?: string;
  page?: number;
  ordering?: string;
}

// ===== Wine/Proton Management =====

/**
 * GE-Proton release from GitHub
 */
export interface ProtonRelease {
  tag_name: string;     // Version tag (e.g., "GE-Proton9-21")
  name: string;         // Release name
  published_at: string; // ISO timestamp
  download_url: string; // Download URL
  size: number;         // File size in bytes
}

// ===== Helper Types =====

/**
 * Command result wrapper
 */
export interface CommandResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}
