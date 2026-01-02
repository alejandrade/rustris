/**
 * Game Service - Main API for game management
 * Wraps all Tauri commands for game operations
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/**
 * Event payload for real-time log chunks
 */
export interface LogChunkPayload {
  slug: string;
  lines: string[];
  chunk: string;
  total_lines: number;
}

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

  /**
   * Clear the log buffer for a game
   */
  async clearGameLog(slug: string): Promise<void> {
    return invoke("clear_game_log", { slug });
  }

  /**
   * Listen to real-time log events for a specific game
   * Returns an unlisten function to stop listening
   *
   * Usage:
   * ```ts
   * const unlisten = await gameService.onGameLog("game-slug", (payload) => {
   *   console.log("New log chunk:", payload.chunk);
   *   // Append to your log view
   * });
   *
   * // Later: stop listening
   * unlisten();
   * ```
   */
  async onGameLog(
    slug: string,
    callback: (payload: LogChunkPayload) => void
  ): Promise<UnlistenFn> {
    return listen<LogChunkPayload>(`game-log:${slug}`, (event) => {
      callback(event.payload);
    });
  }
}

export const gameService = new GameService();
export default gameService;
