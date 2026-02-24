import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);

// Skryje splash screen hned po mountu Reactu
const splash = document.getElementById("splash");
if (splash) {
  splash.classList.add("hidden");
  setTimeout(() => splash.remove(), 350);
}
