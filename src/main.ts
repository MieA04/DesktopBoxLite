import { App } from "./components/App";
import "./styles/default.css";

// Wait for DOM to be ready, then initialize the app
document.addEventListener("DOMContentLoaded", () => {
  const appElement = document.getElementById("app");
  if (!appElement) {
    console.error("App root element not found");
    return;
  }

  // Insert the app HTML structure
  appElement.innerHTML = `
    <div class="app-container">
      <div class="resizable-window">
        <div class="drag-handle" data-tauri-drag-region></div>
        <div class="search-bar"></div>
        <div class="icon-grid"></div>
      </div>
    </div>
  `;

  // Initialize the app
  new App();
});
