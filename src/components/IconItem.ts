import { type IconInfo } from "../utils/types";
import { openFilePath } from "../utils/icons";

/** Creates a single icon element for the grid. */
export function createIconItem(
  icon: IconInfo,
  onClick?: (path: string) => void,
): HTMLElement {
  const item = document.createElement("div");
  item.className = "icon-item";
  item.title = icon.path;

  const img = document.createElement("img");
  img.className = "icon-image";
  img.alt = icon.name;
  img.draggable = false;

  // Use base64 icon data from backend if available
  if (icon.icon_data) {
    img.src = `data:image/png;base64,${icon.icon_data}`;
  } else {
    // Fallback: hide the image element (show only text)
    img.style.display = "none";
  }

  const label = document.createElement("span");
  label.className = "icon-label";
  label.textContent = icon.name;

  item.appendChild(img);
  item.appendChild(label);

  // Click: increment click count (fire-and-forget) + open file
  item.addEventListener("click", async () => {
    if (onClick) {
      onClick(icon.path);
    }
    await openFilePath(icon.path);
  });

  return item;
}
