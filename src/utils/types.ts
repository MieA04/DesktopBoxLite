/** Desktop icon information from the Rust backend. */
export interface IconInfo {
  name: string;
  path: string;
  icon_data: string;
  is_shortcut: boolean;
  click_count: number;
}

/** Application configuration structure. */
export interface Config {
  hotkeys: HotkeyConfig;
  appearance: AppearanceConfig;
  behavior: BehaviorConfig;
}

export interface HotkeyConfig {
  toggle_window: string;
  custom_commands: CustomCommand[];
}

export interface CustomCommand {
  keys: string;
  command: string;
  description?: string;
}

export interface AppearanceConfig {
  css_path: string | null;
}

export interface BehaviorConfig {
  auto_start: boolean;
  icon_refresh_interval_ms: number;
  window_width: number;
  window_height: number;
}

/** Window resize direction. */
export type ResizeDirection = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";
