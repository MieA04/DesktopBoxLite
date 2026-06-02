import { type IconInfo } from "./types";
import { invoke } from "@tauri-apps/api/core";

/** Maximum number of icons per row. */
export const MAX_ICONS_PER_ROW = 10;

/** Icon width in pixels (matches CSS .icon-item width). */
export const ICON_WIDTH = 120;

/** Number of top-clicked icons shown in the "Frequently Used" section. */
export const FREQUENT_ICON_COUNT = 10;

/** Fetches desktop icons from the Rust backend. */
export async function fetchIcons(): Promise<IconInfo[]> {
  try {
    const icons = await invoke<IconInfo[]>("get_icons");
    return icons;
  } catch (error) {
    console.error("Failed to fetch icons:", error);
    return [];
  }
}

/** Filters icons by a search query (case-insensitive, fuzzy). */
export function filterIcons(icons: IconInfo[], query: string): IconInfo[] {
  if (!query.trim()) {
    return icons;
  }

  const lowerQuery = query.toLowerCase().trim();
  return icons.filter((icon) => icon.name.toLowerCase().includes(lowerQuery));
}

/** Opens a file or folder at the given path via the Rust backend. */
export async function openFilePath(path: string): Promise<void> {
  try {
    await invoke("open_file", { path });
  } catch (error) {
    console.error("Failed to open file:", error);
  }
}

