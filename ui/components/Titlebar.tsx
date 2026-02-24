import { useNavigate, useLocation } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-react";

const NAV_LINKS = [
  { label: "Leaderboard", path: "/" },
  { label: "Matches", path: "/match" },
  { label: "About", path: "/settings" },
];

export default function Titlebar() {
  const navigate = useNavigate();
  const location = useLocation();

  const handleMinimize = () => getCurrentWindow().minimize();
  const handleClose = () => getCurrentWindow().close();

  return (
    <header className="flex items-center justify-between h-12 px-4 border-b border-[var(--border)] bg-[var(--bg-primary)] select-none shrink-0">

      {/* Drag region — jen střed, NE přes buttony */}
      <div data-tauri-drag-region className="absolute inset-0 h-12" style={{ zIndex: 0 }} />

      {/* Logo */}
      <div
        className="relative z-10 flex items-center gap-2 cursor-pointer"
        onClick={() => navigate("/")}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" className="text-[var(--accent)]">
          <path d="M4 4L12 20L14 16L10 8H16L20 4H4Z" fill="currentColor" />
        </svg>
        <span className="text-sm font-bold tracking-[0.2em] text-white">VANTAGE</span>
      </div>

      {/* Nav */}
      <nav className="relative z-10 flex items-center gap-6">
        {NAV_LINKS.map((link) => (
          <button
            key={link.path}
            onClick={() => navigate(link.path)}
            className={`text-sm transition-colors ${
              location.pathname === link.path
                ? "text-white font-medium"
                : "text-[var(--text-secondary)] hover:text-white"
            }`}
          >
            {link.label}
          </button>
        ))}
      </nav>

      {/* Window Controls */}
      <div className="relative z-10 flex items-center gap-0.5">
        <button
          onClick={handleMinimize}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-white transition-colors"
        >
          <Minus size={13} />
        </button>
        <button
          onClick={handleClose}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-[var(--accent)] text-[var(--text-muted)] hover:text-white transition-colors"
        >
          <X size={13} />
        </button>
      </div>
    </header>
  );
}
