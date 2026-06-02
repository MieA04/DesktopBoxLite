import { getCurrentWindow, LogicalSize, LogicalPosition } from "@tauri-apps/api/window";
import { type ResizeDirection } from "../utils/types";

/** Minimum window size in pixels. */
const MIN_WIDTH = 120;
const MIN_HEIGHT = 150;

/**
 * Creates resize handles on all edges and corners of the window.
 *
 * Unlike the previous CSS-only approach, this resizes the **OS window**
 * itself via Tauri's `Window.setSize()` / `Window.setPosition()` APIs.
 * The `.resizable-window` div always fills 100% of the Tauri window,
 * so changing the window size automatically reflows the flex layout.
 */
export class ResizeHandle {
  private target: HTMLElement;
  private onResize: (width: number, height: number) => void;
  private handles: HTMLElement[] = [];

  constructor(
    target: HTMLElement,
    onResize: (width: number, height: number) => void
  ) {
    this.target = target;
    this.onResize = onResize;
    this.createHandles();
  }

  /** Creates resize handle elements for all edges and corners. */
  private createHandles(): void {
    const directions: ResizeDirection[] = ["n", "s", "e", "w", "ne", "nw", "se", "sw"];

    for (const dir of directions) {
      const handle = document.createElement("div");
      handle.className = `resize-handle resize-handle-${dir}`;
      this.target.appendChild(handle);
      this.handles.push(handle);

      this.addDragListener(handle, dir);
    }
  }

  /** Adds mouse drag listeners for resizing via Tauri Window API. */
  private addDragListener(handle: HTMLElement, direction: ResizeDirection): void {
    let isDragging = false;
    let startX: number;
    let startY: number;
    let startWidth: number;
    let startHeight: number;
    let startWindowX: number;
    let startWindowY: number;

    const appWindow = getCurrentWindow();

    const onMouseDown = async (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();

      isDragging = true;
      startX = e.clientX;
      startY = e.clientY;
      startWidth = window.innerWidth;
      startHeight = window.innerHeight;

      // Capture current window position for W/N direction adjustments
      try {
        const pos = await appWindow.outerPosition();
        startWindowX = pos.x;
        startWindowY = pos.y;
      } catch {
        startWindowX = 0;
        startWindowY = 0;
      }

      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    };

    const onMouseMove = async (e: MouseEvent) => {
      if (!isDragging) return;

      const dx = e.clientX - startX;
      const dy = e.clientY - startY;

      let newWidth = startWidth;
      let newHeight = startHeight;
      let newX = startWindowX;
      let newY = startWindowY;

      // Calculate new size based on drag direction
      if (direction.includes("e")) {
        newWidth = Math.max(MIN_WIDTH, startWidth + dx);
      }
      if (direction.includes("w")) {
        newWidth = Math.max(MIN_WIDTH, startWidth - dx);
        // Shift window right to keep right edge in place when expanding left
        const widthDelta = newWidth - startWidth;
        newX = startWindowX - widthDelta;
      }
      if (direction.includes("s")) {
        newHeight = Math.max(MIN_HEIGHT, startHeight + dy);
      }
      if (direction.includes("n")) {
        newHeight = Math.max(MIN_HEIGHT, startHeight - dy);
        // Shift window down to keep bottom edge in place when expanding upward
        const heightDelta = newHeight - startHeight;
        newY = startWindowY - heightDelta;
      }

      // Apply position change first (W/N directions), then size
      try {
        if (newX !== startWindowX || newY !== startWindowY) {
          await appWindow.setPosition(new LogicalPosition(newX, newY));
        }
        await appWindow.setSize(new LogicalSize(newWidth, newHeight));
      } catch (err) {
        console.error("Window resize failed:", err);
      }

      // Notify caller for persistence
      this.onResize(newWidth, newHeight);
    };

    const onMouseUp = () => {
      isDragging = false;
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      // Persist final size
      this.onResize(window.innerWidth, window.innerHeight);
    };

    handle.addEventListener("mousedown", onMouseDown);
  }

  /** Removes all resize handles from the DOM. */
  destroy(): void {
    for (const handle of this.handles) {
      handle.remove();
    }
    this.handles = [];
  }
}
