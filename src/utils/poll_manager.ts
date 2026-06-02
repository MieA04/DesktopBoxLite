import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { IconInfo } from "./types";

/**
 * Polls for desktop icon changes every `intervalMs` and triggers
 * background extraction when a change is detected.
 *
 * Architecture:
 *   poll() → check_icons_changed()  (轻量指纹对比, ~5ms)
 *              │
 *              ▼  changed?
 *            / \
 *          Yes  No → wait for next poll
 *           │
 *           ▼
 *     renderGuard: isRefreshing?
 *           │
 *        /       \
 *      Yes        No
 *       │          │
 *     mark       invoke("refresh_icons")  (立即返回)
 *   pending       │
 *                 ▼
 *           wait "icons-ready" event  (后台提取完成后触发)
 *                 │
 *                 ▼
 *           render icons
 *           isRefreshing = false
 *                 │
 *           if pendingRefresh → next poll picks it up
 */
export class PollManager {
  private intervalMs: number;
  private onIconsUpdate: (icons: IconInfo[]) => void;
  private onError: (error: string) => void;
  private timer: ReturnType<typeof setInterval> | null = null;
  private isRefreshing = false;
  private pendingRefresh = false;

  constructor(
    intervalMs: number,
    onIconsUpdate: (icons: IconInfo[]) => void,
    onError?: (error: string) => void,
  ) {
    this.intervalMs = intervalMs;
    this.onIconsUpdate = onIconsUpdate;
    this.onError = onError ?? console.error;
  }

  /** Starts the polling loop. Does an immediate check on call. */
  start(): void {
    this.poll();
    this.timer = setInterval(() => this.poll(), this.intervalMs);
  }

  /** Stops the polling loop. */
  stop(): void {
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  /** Triggers an immediate out-of-cycle refresh (e.g. from tray "reload"). */
  forceRefresh(): void {
    if (this.isRefreshing) {
      this.pendingRefresh = true;
    } else {
      this.doRefresh();
    }
  }

  // ── Private ──────────────────────────────────

  private async poll(): Promise<void> {
    try {
      const changed = await invoke<boolean>("check_icons_changed");
      if (!changed) return;

      // Render guard: 如果已经在刷新，标记等待
      if (this.isRefreshing) {
        this.pendingRefresh = true;
        return;
      }

      await this.doRefresh();
    } catch (err) {
      this.onError(`Poll check failed: ${err}`);
    }
  }

  private async doRefresh(): Promise<void> {
    this.isRefreshing = true;

    try {
      // Step 1: 异步触发后台提取（立即返回）
      await invoke("refresh_icons");

      // Step 2: 等待 "icons-ready" 事件
      const icons = await this.waitForReady();

      if (icons.length > 0) {
        this.onIconsUpdate(icons);
      }
    } catch (err) {
      this.onError(`Icon refresh failed: ${err}`);
    } finally {
      this.isRefreshing = false;

      // 如果在提取过程中又有新的变更被标记，下一轮 poll 会处理
      if (this.pendingRefresh) {
        this.pendingRefresh = false;
        // 不立即触发——等下一轮定时 poll，避免频繁提取
      }
    }
  }

  /** Returns a Promise that resolves when the "icons-ready" event fires. */
  private waitForReady(): Promise<IconInfo[]> {
    return new Promise((resolve) => {
      let unlistenFn: (() => void) | null = null;

      listen<IconInfo[]>("icons-ready", (event) => {
        if (unlistenFn) unlistenFn();
        resolve(event.payload);
      }).then((fn) => {
        unlistenFn = fn;
      });

      // Safety timeout: 30s, prevents infinite waiting
      setTimeout(() => {
        if (unlistenFn) unlistenFn();
        resolve([]);
      }, 30000);
    });
  }
}
