import { type IconInfo } from "../utils/types";
import { createIconItem } from "./IconItem";
import { MAX_ICONS_PER_ROW, FREQUENT_ICON_COUNT } from "../utils/icons";

/** Manages the icon grid container with responsive layout and sectioned display. */
export class IconGrid {
  private container: HTMLElement;
  private currentIcons: IconInfo[] = [];
  private onIconClick: (path: string) => void;

  constructor(
    container: HTMLElement,
    onIconClick: (path: string) => void,
  ) {
    this.container = container;
    this.onIconClick = onIconClick;
    this.container.style.setProperty("--max-columns", String(MAX_ICONS_PER_ROW));
  }

  /** Renders icons — optionally split into "Frequently Used" and "All Apps" sections. */
  render(icons: IconInfo[], sectioned: boolean = false): void {
    this.currentIcons = icons;
    this.container.innerHTML = "";

    if (!sectioned) {
      // Flat list during search
      for (const icon of icons) {
        this.container.appendChild(createIconItem(icon, this.onIconClick));
      }
      return;
    }

    // ── Sectioned display ──────────────────────────────
    const frequent = this.getFrequentIcons(icons);
    const allApps = [...icons].sort((a, b) =>
      a.name.localeCompare(b.name),
    );

    if (frequent.length > 0) {
      this.addSectionHeader("常用应用");
      for (const icon of frequent) {
        this.container.appendChild(createIconItem(icon, this.onIconClick));
      }
    }

    this.addSectionHeader("所有应用");
    for (const icon of allApps) {
      this.container.appendChild(createIconItem(icon, this.onIconClick));
    }
  }

  /**
   * Returns the top `FREQUENT_ICON_COUNT` icons by click count.
   * Edge cases:
   * - All zero clicks → returns empty (no frequent section shown)
   * - Fewer than 10 with clicks → returns whatever has clicks
   * - Ties → sorted by name for deterministic ordering
   */
  private getFrequentIcons(icons: IconInfo[]): IconInfo[] {
    return icons
      .filter((i) => i.click_count > 0)
      .sort((a, b) => {
        if (b.click_count !== a.click_count) {
          return b.click_count - a.click_count;
        }
        return a.name.localeCompare(b.name);
      })
      .slice(0, FREQUENT_ICON_COUNT);
  }

  /** Appends a section header element spanning the full grid width. */
  private addSectionHeader(title: string): void {
    const header = document.createElement("div");
    header.className = "section-header";
    header.textContent = title;
    this.container.appendChild(header);
  }

  /** Returns the currently displayed icons. */
  getCurrentIcons(): IconInfo[] {
    return this.currentIcons;
  }

  /** Clears all icons from the grid. */
  clear(): void {
    this.currentIcons = [];
    this.container.innerHTML = "";
  }
}
