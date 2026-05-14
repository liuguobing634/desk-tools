import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "$lib/types/workbench";

export function getSettings() {
  return invoke<AppSettings>("get_settings");
}

export function updateSettings(settings: AppSettings) {
  return invoke<string>("update_settings", { settings });
}
