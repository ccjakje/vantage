import { useState, useEffect, useCallback, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Search } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface RecentSearch {
  name: string;
  tag: string;
  rank: string;
}

type ConnectionStatus = "connecting" | "connected" | "disconnected";

export default function Home() {
  const [query, setQuery] = useState("");
  const [phase, setPhase] = useState("menu");
  const [status, setStatus] = useState<ConnectionStatus>("connecting");
  const [recentSearches] = useState<RecentSearch[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();
  const prevPhase = useRef("menu");

  // Init klient při mountu
  useEffect(() => {
    invoke("init_client")
      .then(() => setStatus("connected"))
      .catch(() => setStatus("disconnected"));
  }, []);

  // Poll fáze každé 3s
  const pollPhase = useCallback(async () => {
    try {
      const p = await invoke<string>("get_current_phase");
      setPhase(p);
      setStatus("connected");

      // Auto-navigate do match view když se detekuje hra
      if (p !== "menu" && prevPhase.current === "menu") {
        navigate("/match");
      }
      prevPhase.current = p;
    } catch {
      setStatus("disconnected");
    }
  }, [navigate]);

  useEffect(() => {
    if (status === "disconnected") return;
    pollPhase();
    const interval = setInterval(pollPhase, 3000);
    return () => clearInterval(interval);
  }, [status, pollPhase]);

  // "/" shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "/" && document.activeElement !== inputRef.current) {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleSearch = useCallback(() => {
    const parts = query.trim().split("#");
    if (parts.length === 2 && parts[0].trim() && parts[1].trim()) {
      console.log("Search:", parts[0].trim(), parts[1].trim());
    }
  }, [query]);

  // Status dot config
  const statusConfig = {
    connecting: { color: "bg-yellow-500", pulse: true,  label: "Connecting to Valorant..." },
    connected:  { color: "bg-emerald-500", pulse: false, label: "Connected" },
    disconnected: { color: "bg-red-500",  pulse: false, label: "Valorant not detected" },
  }[status];

  return (
    <div className="flex flex-col min-h-full">
      <div className="flex-1 flex flex-col items-center justify-center px-6 py-10">

        {/* ── Connection indicator ── */}
        <div className="mb-8 flex items-center gap-2">
          <span className="relative flex h-2 w-2">
            {statusConfig.pulse && (
              <span className={`animate-ping absolute inline-flex h-full w-full rounded-full ${statusConfig.color} opacity-75`} />
            )}
            <span className={`relative inline-flex rounded-full h-2 w-2 ${statusConfig.color}`} />
          </span>
          <span className="text-xs text-[var(--text-muted)]">{statusConfig.label}</span>

          {/* Retry pokud odpojeno */}
          {status === "disconnected" && (
            <button
              onClick={() => {
                setStatus("connecting");
                invoke("init_client")
                  .then(() => setStatus("connected"))
                  .catch(() => setStatus("disconnected"));
              }}
              className="text-[10px] text-[var(--accent)] hover:underline ml-1"
            >
              Retry
            </button>
          )}
        </div>

        {/* ── Live match banner ── */}
        <AnimatePresence>
          {phase !== "menu" && status === "connected" && (
            <motion.div
              initial={{ opacity: 0, y: -12, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -12, scale: 0.97 }}
              className="mb-6"
            >
              <button
                onClick={() => navigate("/match")}
                className="flex items-center gap-3 px-5 py-3 rounded-lg border border-emerald-500/30 bg-emerald-500/10 hover:bg-emerald-500/15 hover:border-emerald-500/50 transition-all group"
              >
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-400" />
                </span>
                <span className="text-sm text-emerald-300 font-medium">
                  {phase === "pregame" ? "Agent Select in progress" : "Match in progress"}
                </span>
                <span className="text-xs text-[var(--text-muted)] group-hover:text-white transition-colors ml-1">
                  View →
                </span>
              </button>
            </motion.div>
          )}
        </AnimatePresence>

        {/* ── Search bar ── */}
        <div className="w-full max-w-xl">
          <div className="relative group">
            <Search
              size={15}
              className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] group-focus-within:text-[var(--text-secondary)] transition-colors z-10"
            />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder="Player Name #Tag"
              className="w-full bg-[var(--bg-input)] border border-[var(--border)] rounded-lg text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--border-light)] transition-colors"
              style={{ height: "52px", paddingLeft: "34px", paddingRight: "40px" }}
            />
            <div className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] text-xs border border-[var(--border)] rounded px-1.5 py-0.5 leading-none">
              /
            </div>
          </div>
          <p className="text-center text-xs text-[var(--text-muted)] mt-2">
            Press Enter to search directly
          </p>
        </div>

        {/* ── Recent Searches ── */}
        {recentSearches.length > 0 && (
          <div className="w-full max-w-xl mt-8">
            <h3 className="text-[10px] font-semibold tracking-[0.15em] text-[var(--text-muted)] mb-3 uppercase">
              Recent Searches
            </h3>
            <div className="space-y-1">
              {recentSearches.map((r, i) => (
                <button
                  key={i}
                  className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-[var(--bg-card)] transition-colors text-left group"
                >
                  <div className="w-8 h-8 rounded-md bg-[var(--bg-card-hover)] flex items-center justify-center text-xs font-bold text-[var(--text-secondary)] shrink-0">
                    {r.name[0]?.toUpperCase()}
                  </div>
                  <div className="min-w-0">
                    <p className="text-sm font-medium text-white leading-tight">{r.name}</p>
                    <p className="text-xs text-[var(--text-muted)] leading-tight mt-0.5">
                      #{r.tag} · {r.rank}
                    </p>
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* ── Footer ── */}
      <footer className="shrink-0 pb-5 text-center">
        <div className="flex items-center justify-center gap-3 text-[10px] text-[var(--text-muted)] mb-1.5">
          <span className="hover:text-[var(--text-secondary)] cursor-pointer transition-colors">Privacy</span>
          <span className="text-[var(--border)]">|</span>
          <span className="hover:text-[var(--text-secondary)] cursor-pointer transition-colors">Terms</span>
          <span className="text-[var(--border)]">|</span>
          <span className="hover:text-[var(--text-secondary)] cursor-pointer transition-colors">API</span>
        </div>
        <p className="text-[10px] text-[var(--text-muted)]">Vantage © 2025. Not affiliated with Riot Games.</p>
      </footer>
    </div>
  );
}
