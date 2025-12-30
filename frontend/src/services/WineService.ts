/**
 * Wine Service - Wine/Proton version management
 */

import { invoke } from "@tauri-apps/api/core";

export interface WineVersionInfo {
  path: string;
  display_name: string;
}

export interface GeProtonRelease {
  tag_name: string;
  name: string;
  published_at: string;
  download_url: string;
  size_mb: number;
}

class WineService {
  /**
   * Get list of installed Wine/Proton versions
   */
  async getAvailableVersions(): Promise<WineVersionInfo[]> {
    return invoke<WineVersionInfo[]>("get_available_wine_versions");
  }

  /**
   * Set Lutris's global default Wine version for all new games
   */
  async setGlobalDefaultVersion(winePath: string): Promise<void> {
    return invoke("set_lutris_global_default_wine_version", { winePath });
  }

  /**
   * Get Lutris's global default Wine version
   */
  async getGlobalDefaultVersion(): Promise<string | null> {
    return invoke<string | null>("get_lutris_global_default_wine_version");
  }

  /**
   * Delete an installed Proton version
   */
  async deleteProtonVersion(path: string): Promise<void> {
    return invoke("delete_proton_version", { path });
  }

  /**
   * Fetch available GE-Proton releases from GitHub
   */
  async fetchGeProtonReleases(): Promise<GeProtonRelease[]> {
    return invoke<GeProtonRelease[]>("fetch_ge_proton_releases");
  }

  /**
   * Download and install a GE-Proton version
   */
  async downloadGeProton(tagName: string, downloadUrl: string): Promise<string> {
    return invoke<string>("download_ge_proton", { tagName, downloadUrl });
  }
}

export const wineService = new WineService();
export default wineService;
