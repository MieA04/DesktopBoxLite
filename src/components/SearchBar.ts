/** Manages the search bar input with real-time filtering callback. */
export class SearchBar {
  private input: HTMLInputElement;
  private onSearch: (query: string) => void;

  constructor(container: HTMLElement, onSearch: (query: string) => void) {
    this.onSearch = onSearch;

    this.input = document.createElement("input");
    this.input.type = "text";
    this.input.placeholder = "搜索图标...";
    this.input.className = "search-input";

    // Debounced search on input
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    this.input.addEventListener("input", () => {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      debounceTimer = setTimeout(() => {
        this.onSearch(this.input.value);
      }, 50);
    });

    container.appendChild(this.input);
  }

  /** Clears the search input and triggers a reset. */
  clear(): void {
    this.input.value = "";
    this.onSearch("");
  }

  /** Returns the current search query. */
  getQuery(): string {
    return this.input.value;
  }

  /** Focuses the search input. */
  focus(): void {
    this.input.focus();
  }
}
