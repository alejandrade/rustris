/**
 * Lutris Service - Lutris.net API integration
 * For browsing and searching games on Lutris.net
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  LutrisGameListResponse,
  LutrisSearchParams,
} from "../types/lutris";

class LutrisService {
  /**
   * Search/browse games on Lutris.net
   */
  async searchGames(params: LutrisSearchParams = {}): Promise<LutrisGameListResponse> {
    return invoke<LutrisGameListResponse>("search_lutris_games", {
      search: params.search || null,
    });
  }

  /**
   * Get installers for a specific game
   */
  async getInstallers(gameSlug: string): Promise<any[]> {
    return invoke<any[]>("get_lutris_installers", {
      gameSlug,
    });
  }

  /**
   * Run a Lutris installer from YAML content
   */
  async runInstallerFromYaml(yamlContent: string, gameName: string): Promise<string> {
    return invoke<string>("run_lutris_installer_from_yaml", {
      yamlContent,
      gameName,
    });
  }

}

export const lutrisService = new LutrisService();
export default lutrisService;
