/**
 * Game Service - Main API for game management
 * Wraps all Tauri commands for game operations
 */

import { invoke } from "@tauri-apps/api/core";

export interface Game {
  slug: string;
  name: string;
  runner: string | null;
  directory: string | null;
  playtime: number;
  last_played: string | null;
  executable: string | null;
  wine_version: string | null;
  wine_prefix: string | null;
  environment_vars: string | null;
  cover_url: string | null;
  debug_output: boolean;
}

class GameService {
  /**
   * Get all games
   */
  async getGames(): Promise<Game[]> {
    return invoke<Game[]>("get_games");
  }

  /**
   * Launch a game by its slug
   */
  async launchGame(slug: string): Promise<void> {
    return invoke("launch_game_by_slug", { slug });
  }

  /**
   * Check if a game is currently running
   */
  async checkGameRunning(slug: string): Promise<boolean> {
    return invoke<boolean>("check_game_running", { slug });
  }

  /**
   * Get game log content
   */
  async getGameLog(slug: string): Promise<string> {
    return invoke<string>("get_game_log", { slug });
  }

  /**
   * Save game log to downloads directory
   */
  async saveGameLog(slug: string): Promise<string> {
    return invoke<string>("save_game_log", { slug });
  }

  /**
   * Update the Wine/Proton version for a game
   */
  async updateWineVersion(slug: string, wineVersion: string): Promise<void> {
    return invoke("update_game_wine_version", { slug, wineVersion });
  }
}

export const gameService = new GameService();
export default gameService;
