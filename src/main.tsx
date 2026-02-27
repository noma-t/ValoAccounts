import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { ShopWindow } from "./pages/ShopWindow";

function getShopAccountId(): number | null {
  try {
    const label = getCurrentWindow().label
    if (label.startsWith('shop-')) {
      const parsed = parseInt(label.replace('shop-', ''), 10)
      return isNaN(parsed) ? null : parsed
    }
  } catch {
    // Not in a Tauri context or not a shop window
  }
  return null
}

const shopAccountId = getShopAccountId()

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {shopAccountId !== null ? (
      <ShopWindow accountId={shopAccountId} />
    ) : (
      <App />
    )}
  </React.StrictMode>,
);
