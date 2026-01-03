/**
 * Utility Service - Miscellaneous and testing commands
 */
import { invoke } from "@tauri-apps/api/core";

export type OpenTarget =
  | { type: 'path'; value: string }
  | { type: 'url'; value: string }
  | { type: 'directory'; value: string };

class UtilityService {
  /**
   * Opens a given target (file, URL, or directory) using the system's default application.
   * @param target The target to open, specified as an object that matches the backend `OpenTarget` enum.
   */
  async open(target: OpenTarget): Promise<void> {
    return invoke("open_target", { target });
  }
}

export const utilityService = new UtilityService();
export default utilityService;
