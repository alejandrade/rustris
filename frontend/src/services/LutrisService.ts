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

}

export const lutrisService = new LutrisService();
export default lutrisService;
