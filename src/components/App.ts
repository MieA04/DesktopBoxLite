import { IconGrid } from "./IconGrid";
import { SearchBar } from "./SearchBar";
import { ResizeHandle } from "./ResizeHandle";
import { filterIcons } from "../utils/icons";
import { PollManager } from "../utils/poll_manager";
import { type IconInfo, type Config } from "../utils/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize, LogicalPosition, primaryMonitor } from "@tauri-apps/api/window";

/** Main application controller. */
export class App {
  private iconGrid: IconGrid;
  private searchBar: SearchBar;
  private pollManager: PollManager;
  private allIcons: IconInfo[] = [];
  private config: Config | null = null;
  private windowElement: HTMLElement;
  // Cached display geometry used by show/hide animations.
  // Populated on first call, avoids repeated async lookups.
  private cachedSw: number = 0;
  private cachedSh: number = 0;

  constructor() {
    const appContainer = document.getElementById("app")!;
    this.windowElement =
      appContainer.querySelector<HTMLElement>(".resizable-window")!;
    const searchContainer =
      appContainer.querySelector<HTMLElement>(".search-bar")!;
    const gridContainer =
      appContainer.querySelector<HTMLElement>(".icon-grid")!;

    this.iconGrid = new IconGrid(gridContainer, (path) => {
      this.handleIconClick(path);
    });

    this.searchBar = new SearchBar(searchContainer, (query) => {
      this.handleSearch(query);
    });

    new ResizeHandle(this.windowElement, (width, height) => {
      this.handleResize(width, height);
    });

    this.pollManager = new PollManager(
      500,
      (icons) => {
        this.allIcons = icons;
        this.handleSearch(this.searchBar.getQuery());
      },
      (error) => console.error("PollManager error:", error),
    );
    this.pollManager.start();

    this.loadConfig().then(() => {
      this.applyWindowSize();
      this.applyCustomCss();
    });

    this.setupAnimationListeners();
  }

  /**
   * Ensures the display geometry cache is populated.
   * Reads the primary monitor size once and stores it.
   */
  private async ensureDisplayCache(): Promise<void> {
    if (this.cachedSh > 0) return;
    const monitor = await primaryMonitor();
    if (monitor) {
      this.cachedSw = monitor.size.width;
      this.cachedSh = monitor.size.height;
    }
  }

  /**
   * Returns the target window width/height from config (if loaded)
   * or falls back to the tauri.conf.json defaults.
   */
  private getWindowSize(): { width: number; height: number } {
    if (this.config) {
      return {
        width: this.config.behavior.window_width,
        height: this.config.behavior.window_height,
      };
    }
    // Hardcoded defaults matching tauri.conf.json
    return { width: 800, height: 600 };
  }

  /** Listens for Tauri events that trigger show/hide animations. */
  private setupAnimationListeners(): void {
    // ── Hide ─────────────────────────────────────────────
    // Smoothly resize the Tauri OS window from full height → minimum.
    // Uses setInterval (not rAF) for reliable animation in all window states.
    listen("animate-hide", () => {
      this.ensureDisplayCache().then(() => {
        if (this.cachedSh <= 0) return;
        const appWindow = getCurrentWindow();
        const { width, height } = this.getWindowSize();
        const sw = this.cachedSw;
        const sh = this.cachedSh;
        const x = Math.round((sw - width) / 2);

        const TOTAL_FRAMES = 6;
        let frame = 0;
        const timer = setInterval(() => {
          frame++;
          const progress = frame / TOTAL_FRAMES;
          // ease-out cubic
          const eased = 1 - Math.pow(1 - progress, 3);
          const h = Math.max(1, Math.round(height * (1 - eased)));
          const y = Math.round(sh - h);

          appWindow.setSize(new LogicalSize(width, h));
          appWindow.setPosition(new LogicalPosition(x, y));

          if (frame >= TOTAL_FRAMES) {
            clearInterval(timer);
            invoke("finish_hide").catch((err) =>
              console.error("finish_hide failed:", err),
            );
          }
        }, 16);
      });
    });

    // ── Show ─────────────────────────────────────────────
    // The OS window was left at minimum size after hide.
    // Show it and grow to full height with setInterval.
    listen("animate-show", () => {
      this.ensureDisplayCache().then(() => {
        if (this.cachedSh <= 0) return;
        const appWindow = getCurrentWindow();
        const { width, height } = this.getWindowSize();
        const sw = this.cachedSw;
        const sh = this.cachedSh;
        const x = Math.round((sw - width) / 2);

        const TOTAL_FRAMES = 6;
        let frame = 0;
        const timer = setInterval(() => {
          frame++;
          const progress = frame / TOTAL_FRAMES;
          const eased = 1 - Math.pow(1 - progress, 3);
          const h = Math.round(height * eased);
          const y = Math.round(sh - h);

          appWindow.setSize(new LogicalSize(width, h));
          appWindow.setPosition(new LogicalPosition(x, y));

          if (frame >= TOTAL_FRAMES) {
            clearInterval(timer);
          }
        }, 16);
      });
    });
  }

  /** Loads configuration from the backend. */
  private async loadConfig(): Promise<void> {
    try {
      this.config = await invoke<Config>("get_config");
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  }

  /** Applies the saved window size from config and positions it at bottom-center. */
  private async applyWindowSize(): Promise<void> {
    if (!this.config) return;

    const { window_width, window_height } = this.config.behavior;
    if (window_width <= 0 || window_height <= 0) return;

    const appWindow = getCurrentWindow();

    try {
      // Position at bottom-center of the primary monitor
      const monitor = await primaryMonitor();
      if (monitor) {
        const sw = monitor.size.width;
        const sh = monitor.size.height;
        const x = Math.round((sw - window_width) / 2);
        const y = Math.round(sh - window_height);
        await appWindow.setPosition(new LogicalPosition(x, y));
      }

      // Set the OS window to the saved dimensions
      await appWindow.setSize(new LogicalSize(window_width, window_height));
    } catch (err) {
      console.error("Failed to set window size/position:", err);
    }
  }

  /** Loads custom CSS from the path specified in config. */
  private applyCustomCss(): void {
    const cssPath = this.config?.appearance.css_path;
    if (!cssPath) return;

    const existing = document.getElementById("custom-css");
    if (existing) existing.remove();

    const link = document.createElement("link");
    link.id = "custom-css";
    link.rel = "stylesheet";
    link.href = `file:///${cssPath.replace(/\\/g, "/")}`;
    link.onload = () => console.log("Custom CSS loaded from:", cssPath);
    link.onerror = () => console.warn("Failed to load custom CSS from:", cssPath);
    document.head.appendChild(link);
  }

  /** Handles icon click: increment backend count asynchronously. */
  private handleIconClick(path: string): void {
    // Fire-and-forget: don't block file opening
    invoke<number>("increment_click_count", { path })
      .then((newCount) => {
        // Optimistically update the local click count
        const icon = this.allIcons.find((i) => i.path === path);
        if (icon) {
          icon.click_count = newCount;
        }
        // Re-render sections immediately with the updated count
        this.handleSearch(this.searchBar.getQuery());
      })
      .catch((err) => {
        console.error("Failed to increment click count:", err);
      });
  }

  /** Handles search query changes. */
  private handleSearch(query: string): void {
    const filtered = filterIcons(this.allIcons, query);
    const isSearching = query.trim().length > 0;
    this.iconGrid.render(filtered, !isSearching);
  }

  /** Handles window resize events and persists the size. */
  private handleResize(width: number, height: number): void {
    if (this.resizeSaveTimer) {
      clearTimeout(this.resizeSaveTimer);
    }
    this.resizeSaveTimer = setTimeout(() => {
      invoke("save_window_size", { width, height }).catch((err) => {
        console.error("Failed to save window size:", err);
      });
    }, 500);
  }

  private resizeSaveTimer: ReturnType<typeof setTimeout> | null = null;
}
