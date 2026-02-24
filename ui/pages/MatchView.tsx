import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, Shield, RefreshCw, Clock, TrendingUp } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { AgentIcon, RankInline, RankIcon } from "../components/Icons";
import { rankColor, mapName } from "../utils/icons";

// ── Types ─────────────────────────────────────────────────────────

interface PlayerRow {
  name: string; tag: string; agent: string;
  rank: number; rank_name: string;
  wins: number; total: number; winrate: number;
  hs: number; level: number;
  peak_rank: number; peak_rank_name: string; peak_act: string;
  is_self: boolean; smurf_score: number; smurf_label: string | null;
}

interface MatchData {
  phase: string;
  blue_team: PlayerRow[]; red_team: PlayerRow[];
  blue_avg_rank: string; red_avg_rank: string;
  map: string | null; mode: string | null;
}

interface HistoryEntry {
  match_id: string; map: string; mode: string;
  won: boolean; rounds_won: number; rounds_lost: number;
  kills: number; deaths: number; assists: number;
  acs: number; hs_percent: number; agent: string;
  rank_after: number; rank_after_name: string;
  start_time: number; duration_minutes: number;
}

// ── Live match helpers ─────────────────────────────────────────────

function WinrateBar({ wins, total }: { wins: number; total: number }) {
  if (total === 0) return <span className="text-xs text-[var(--text-muted)]">N/A</span>;
  const pct = (wins / total) * 100;
  const color = pct >= 55 ? "var(--green)" : pct >= 45 ? "var(--text-secondary)" : "var(--red)";
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-xs font-medium" style={{ color }}>{pct.toFixed(0)}%</span>
      <div className="w-16 h-1 rounded-full bg-[var(--border)]">
        <div className="h-full rounded-full" style={{ width: `${Math.min(pct, 100)}%`, background: color }} />
      </div>
      <span className="text-[10px] text-[var(--text-muted)]">{wins}W {total - wins}L</span>
    </div>
  );
}

function PlayerCard({ player, index }: { player: PlayerRow; index: number }) {
  const hsColor = player.hs >= 25 ? "var(--green)" : player.hs >= 15 ? "var(--text-secondary)" : player.hs > 0 ? "var(--red)" : "var(--text-muted)";
  return (
    <motion.div
      initial={{ opacity: 0, y: 5 }} animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.04 }}
      className={`flex items-center gap-3 px-3 py-2 rounded-lg ${
        player.is_self ? "bg-yellow-500/10 border border-yellow-500/25" : "hover:bg-[var(--bg-card-hover)]"
      }`}
    >
      <AgentIcon name={player.agent} size={30} />
      <div className="w-36 shrink-0 min-w-0">
        <div className={`text-sm font-medium truncate ${player.is_self ? "text-yellow-400" : "text-white"}`}>
          {player.is_self ? "★ " : ""}{player.name || <span className="italic text-[var(--text-muted)]">hidden</span>}
        </div>
        <div className="text-[10px] text-[var(--text-muted)] truncate">
          {player.tag ? `#${player.tag}` : player.agent || "—"}
        </div>
      </div>
      <div className="w-28 shrink-0"><RankInline tier={player.rank} size={18} /></div>
      <div className="w-20 shrink-0"><WinrateBar wins={player.wins} total={player.total} /></div>
      <div className="w-12 shrink-0 text-center">
        <span className="text-sm font-medium" style={{ color: hsColor }}>{player.hs > 0 ? `${player.hs.toFixed(0)}%` : "—"}</span>
        <div className="text-[10px] text-[var(--text-muted)]">HS</div>
      </div>
      <div className="w-10 shrink-0 text-center">
        {player.level > 0 && <>
          <div className="text-xs font-medium text-[var(--text-secondary)]">{player.level}</div>
          <div className="text-[10px] text-[var(--text-muted)]">Lv</div>
        </>}
      </div>
      <div className="w-24 shrink-0">
        {player.peak_rank > 0 && (
          <div className="flex items-center gap-1">
            <RankIcon tier={player.peak_rank} size={16} />
            <div>
              <div className="text-[10px] font-medium leading-none" style={{ color: rankColor(player.peak_rank) }}>
                {player.peak_rank_name}
              </div>
              {player.peak_act && <div className="text-[10px] text-[var(--text-muted)] leading-none mt-0.5 truncate max-w-[80px]">{player.peak_act}</div>}
            </div>
          </div>
        )}
      </div>
      <div className="w-16 shrink-0 text-right">
        <AnimatePresence>
          {player.smurf_label && (
            <motion.span initial={{ scale: 0.8, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}
              className={`inline-flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded ${
                player.smurf_label.includes("SMURF")
                  ? "bg-red-500/20 text-red-400 border border-red-500/30"
                  : "bg-yellow-500/20 text-yellow-400 border border-yellow-500/30"
              }`}>
              <AlertTriangle size={9} />
              {player.smurf_label.replace("⚠ ", "").replace("? ", "")}
            </motion.span>
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
}

function TeamSection({ label, players, avgRank, accentColor }: {
  label: string; players: PlayerRow[]; avgRank: string; accentColor: string;
}) {
  if (players.length === 0) return null;
  return (
    <div>
      <div className="flex items-center justify-between mb-2 px-3">
        <div className="flex items-center gap-2">
          <Shield size={13} style={{ color: accentColor }} />
          <span className="text-xs font-bold tracking-wider" style={{ color: accentColor }}>{label}</span>
        </div>
        <span className="text-xs text-[var(--text-muted)]">{avgRank}</span>
      </div>
      <div className="flex items-center gap-3 px-3 mb-1 text-[10px] text-[var(--text-muted)] uppercase tracking-wider">
        <div className="w-[30px] shrink-0" />
        <div className="w-36 shrink-0">Player</div>
        <div className="w-28 shrink-0">Rank</div>
        <div className="w-20 shrink-0">Win Rate</div>
        <div className="w-12 shrink-0 text-center">HS%</div>
        <div className="w-10 shrink-0 text-center">Lv</div>
        <div className="w-24 shrink-0">Peak</div>
        <div className="w-16 shrink-0 text-right">Flag</div>
      </div>
      <div className="space-y-0.5">
        {players.map((p, i) => <PlayerCard key={i} player={p} index={i} />)}
      </div>
    </div>
  );
}

// ── Match history helpers ──────────────────────────────────────────

function formatDate(ms: number): string {
  if (!ms) return "—";
  const d = new Date(ms);
  const months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${months[d.getMonth()]} ${d.getDate()}, ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function timeAgo(ms: number): string {
  const diff = Date.now() - ms;
  const m = Math.floor(diff / 60000);
  const h = Math.floor(m / 60);
  const d = Math.floor(h / 24);
  if (d > 0) return `${d}d ago`;
  if (h > 0) return `${h}h ago`;
  return `${m}m ago`;
}

function modeLabel(mode: string): string {
  const labels: Record<string, string> = {
    competitive: "Competitive", unrated: "Unrated",
    swiftplay: "Swiftplay", spikerush: "Spike Rush",
    deathmatch: "Deathmatch", escalation: "Escalation",
    replication: "Replication", hurm: "Team Deathmatch",
    "skirmish2v2": "2v2 Skirmish",
  };
  return labels[mode.toLowerCase()] ?? mode;
}

function HistorySummaryBar({ entries }: { entries: HistoryEntry[] }) {
  const comp = entries.filter(e => e.mode.toLowerCase() === "competitive");
  if (comp.length === 0) return null;

  const wins = comp.filter(e => e.won).length;
  const totalK = comp.reduce((s, e) => s + e.kills, 0);
  const totalD = comp.reduce((s, e) => s + e.deaths, 0);
  const totalA = comp.reduce((s, e) => s + e.assists, 0);
  const avgHs = comp.reduce((s, e) => s + e.hs_percent, 0) / comp.length;
  const avgAcs = Math.round(comp.reduce((s, e) => s + e.acs, 0) / comp.length);
  const kd = totalD > 0 ? (totalK / totalD).toFixed(2) : totalK.toFixed(0);
  const winPct = Math.round((wins / comp.length) * 100);
  const kdColor = parseFloat(kd) >= 1.2 ? "#4ade80" : parseFloat(kd) >= 0.8 ? "#999" : "#ef4444";

  return (
    <div className="flex items-center gap-6 mb-4 px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
      <div>
        <div className="text-[10px] text-[var(--text-muted)] uppercase tracking-wider mb-0.5">Last {comp.length} Competitive</div>
        <div className="text-xs text-[var(--text-secondary)]">{wins}W / {comp.length - wins}L</div>
      </div>
      <div className="w-px h-8 bg-[var(--border)]" />
      {[
        { label: "K/D", value: kd, color: kdColor },
        { label: "K/D/A", value: `${totalK}/${totalD}/${totalA}`, color: "#fff" },
        { label: "HS%", value: `${avgHs.toFixed(1)}%`, color: avgHs >= 25 ? "#4ade80" : avgHs >= 15 ? "#999" : "#ef4444" },
        { label: "WIN%", value: `${winPct}%`, color: winPct >= 55 ? "#4ade80" : winPct >= 45 ? "#999" : "#ef4444" },
        { label: "AVG ACS", value: avgAcs, color: "#fff" },
      ].map(stat => (
        <div key={stat.label} className="text-center">
          <div className="text-sm font-bold" style={{ color: String(stat.color) }}>{stat.value}</div>
          <div className="text-[10px] text-[var(--text-muted)] mt-0.5">{stat.label}</div>
        </div>
      ))}
    </div>
  );
}

function HistoryRow({ entry, index }: { entry: HistoryEntry; index: number }) {
  const kd = entry.deaths > 0 ? (entry.kills / entry.deaths).toFixed(2) : entry.kills.toFixed(0);
  const kdColor = parseFloat(kd) >= 1.2 ? "#4ade80" : parseFloat(kd) >= 0.8 ? "#999" : "#ef4444";
  const hsColor = entry.hs_percent >= 25 ? "#4ade80" : entry.hs_percent >= 15 ? "#999" : "#ef4444";
  const isComp = entry.mode.toLowerCase() === "competitive";

  return (
    <motion.div
      initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.025 }}
      className={`flex items-center gap-0 rounded-lg overflow-hidden border transition-colors group
        ${entry.won
          ? "border-emerald-500/20 hover:border-emerald-500/40 hover:bg-emerald-500/5"
          : "border-red-500/15 hover:border-red-500/30 hover:bg-red-500/5"
        }`}
    >
      {/* Win/loss accent bar */}
      <div className={`w-[3px] self-stretch shrink-0 ${entry.won ? "bg-emerald-500" : "bg-red-500"}`} />

      {/* Agent icon — large, no border-radius */}
      <div className="shrink-0 w-[52px] h-[52px] bg-[var(--bg-card)] flex items-center justify-center">
        <AgentIcon name={entry.agent} size={38} />
      </div>

      {/* Date + map + mode */}
      <div className="w-[140px] shrink-0 px-3 py-2">
        <div className="text-[10px] text-[var(--text-muted)] mb-0.5">{formatDate(entry.start_time)}</div>
        <div className="text-sm font-semibold text-white leading-tight">{mapName(entry.map)}</div>
        <div className="text-[10px] text-[var(--text-muted)] leading-tight">{modeLabel(entry.mode)}</div>
      </div>

      {/* Score + result badge */}
      <div className="w-[90px] shrink-0 px-2 py-2">
        <div className="flex items-center gap-1 mb-1">
          <span className={`text-base font-bold ${entry.won ? "text-emerald-400" : "text-white"}`}>
            {entry.rounds_won}
          </span>
          <span className="text-[var(--text-muted)] text-sm">:</span>
          <span className={`text-base font-bold ${!entry.won ? "text-red-400" : "text-white"}`}>
            {entry.rounds_lost}
          </span>
        </div>
        <div className={`text-[10px] font-bold px-1.5 py-0.5 rounded inline-block leading-none
          ${entry.won
            ? "bg-emerald-500/20 text-emerald-400"
            : "bg-red-500/20 text-red-400"
          }`}>
          {entry.won ? "WIN" : "LOSS"}
        </div>
      </div>

      {/* KDA */}
      <div className="w-[110px] shrink-0 px-3 py-2">
        <div className="text-sm font-semibold text-white">
          {entry.kills} / <span className="text-[var(--text-muted)]">{entry.deaths}</span> / {entry.assists}
        </div>
        <div className="text-[10px] mt-0.5" style={{ color: kdColor }}>
          K/D {kd}
        </div>
      </div>

      {/* ACS */}
      <div className="w-[70px] shrink-0 px-2 py-2 text-center">
        <div className="text-sm font-semibold text-white">{entry.acs}</div>
        <div className="text-[10px] text-[var(--text-muted)]">ACS</div>
      </div>

      {/* HS% */}
      <div className="w-[60px] shrink-0 px-2 py-2 text-center">
        <div className="text-sm font-semibold" style={{ color: entry.hs_percent > 0 ? hsColor : "var(--text-muted)" }}>
          {entry.hs_percent > 0 ? `${entry.hs_percent.toFixed(0)}%` : "—"}
        </div>
        <div className="text-[10px] text-[var(--text-muted)]">HS%</div>
      </div>

      {/* Rank — only if competitive */}
      <div className="w-[110px] shrink-0 px-3 py-2">
        {isComp && entry.rank_after > 0
          ? <RankInline tier={entry.rank_after} size={18} />
          : <span className="text-[10px] text-[var(--text-muted)]">{modeLabel(entry.mode)}</span>
        }
      </div>

      {/* Duration + time ago */}
      <div className="ml-auto pr-4 text-right shrink-0">
        <div className="text-xs text-[var(--text-secondary)]">{timeAgo(entry.start_time)}</div>
        <div className="text-[10px] text-[var(--text-muted)] mt-0.5">{entry.duration_minutes}m</div>
      </div>
    </motion.div>
  );
}

// ── Main component ─────────────────────────────────────────────────

export default function MatchView() {
  const [liveData, setLiveData] = useState<MatchData | null>(null);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [liveLoading, setLiveLoading] = useState(true);
  const [histLoading, setHistLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [view, setView] = useState<"live" | "history">("live");

  const fetchLive = useCallback(async (silent = false) => {
    if (!silent) setLiveLoading(true); else setRefreshing(true);
    try {
      const d = await invoke<MatchData>("get_match_data");
      setLiveData(d);
      setLastUpdated(new Date());
      // Auto-switch to live view if game detected
      if (d.phase !== "menu") setView("live");
    } catch {}
    finally { setLiveLoading(false); setRefreshing(false); }
  }, []);

  const fetchHistory = useCallback(async () => {
    setHistLoading(true);
    try {
      const h = await invoke<HistoryEntry[]>("get_match_history", { count: 15 });
      setHistory(h);
    } catch {}
    finally { setHistLoading(false); }
  }, []);

  useEffect(() => {
    fetchLive();
    fetchHistory();
    const interval = setInterval(() => fetchLive(true), 10000);
    return () => clearInterval(interval);
  }, [fetchLive, fetchHistory]);

  const isLive = liveData && liveData.phase !== "menu";

  return (
    <div className="flex flex-col h-full">
      {/* ── Toolbar ── */}
      <div className="flex items-center justify-between px-5 pt-4 pb-3 border-b border-[var(--border)] shrink-0">
        <div className="flex items-center gap-1 bg-[var(--bg-card)] rounded-lg p-0.5">
          <button
            onClick={() => setView("live")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
              view === "live"
                ? "bg-[var(--bg-card-hover)] text-white"
                : "text-[var(--text-muted)] hover:text-white"
            }`}
          >
            {isLive && (
              <span className="relative flex h-1.5 w-1.5">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-emerald-400" />
              </span>
            )}
            Live
          </button>
          <button
            onClick={() => setView("history")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
              view === "history"
                ? "bg-[var(--bg-card-hover)] text-white"
                : "text-[var(--text-muted)] hover:text-white"
            }`}
          >
            <TrendingUp size={11} />
            History
          </button>
        </div>

        <div className="flex items-center gap-2">
          {lastUpdated && view === "live" && (
            <span className="text-[10px] text-[var(--text-muted)]">
              Updated {lastUpdated.toLocaleTimeString()}
            </span>
          )}
          <button
            onClick={() => view === "live" ? fetchLive(true) : fetchHistory()}
            disabled={refreshing}
            className="p-1.5 rounded hover:bg-[var(--bg-card-hover)] text-[var(--text-muted)] hover:text-white transition-colors"
          >
            <RefreshCw size={13} className={refreshing ? "animate-spin" : ""} />
          </button>
        </div>
      </div>

      {/* ── Content ── */}
      <div className="flex-1 overflow-y-auto">
        <AnimatePresence mode="wait">

          {/* LIVE VIEW */}
          {view === "live" && (
            <motion.div key="live" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              className="p-5">
              {liveLoading ? (
                <div className="flex items-center justify-center h-48">
                  <div className="flex flex-col items-center gap-3">
                    <div className="w-7 h-7 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" />
                    <span className="text-sm text-[var(--text-muted)]">Fetching player stats...</span>
                  </div>
                </div>
              ) : !isLive ? (
                <div className="flex flex-col items-center justify-center h-48 gap-3">
                  <Shield size={28} className="text-[var(--text-muted)]" />
                  <p className="text-[var(--text-secondary)] font-medium">No active match</p>
                  <p className="text-xs text-[var(--text-muted)]">Start a game in Valorant to see live data</p>
                  <button onClick={() => setView("history")}
                    className="mt-2 text-xs text-[var(--accent)] hover:underline">
                    View match history →
                  </button>
                </div>
              ) : (
                <>
                  <div className="flex items-center gap-3 mb-5">
                    <span className="relative flex h-2 w-2">
                      <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
                        liveData.phase === "pregame" ? "bg-yellow-400" : "bg-emerald-400"}`} />
                      <span className={`relative inline-flex rounded-full h-2 w-2 ${
                        liveData.phase === "pregame" ? "bg-yellow-400" : "bg-emerald-400"}`} />
                    </span>
                    <h2 className="text-sm font-bold">
                      {liveData.phase === "pregame" ? "Agent Select" : "Match In Progress"}
                    </h2>
                    {liveData.map && <span className="text-sm text-[var(--text-muted)]">· {mapName(liveData.map)}</span>}
                    {liveData.mode && <span className="text-xs text-[var(--text-muted)]">{liveData.mode}</span>}
                  </div>
                  <div className="space-y-6">
                    <TeamSection label={liveData.phase === "pregame" ? "YOUR TEAM" : "TEAM BLUE"}
                      players={liveData.blue_team} avgRank={liveData.blue_avg_rank} accentColor="var(--blue)" />
                    {liveData.red_team.length > 0 && <>
                      <div className="border-t border-[var(--border)]" />
                      <TeamSection label="TEAM RED" players={liveData.red_team}
                        avgRank={liveData.red_avg_rank} accentColor="var(--accent)" />
                    </>}
                  </div>
                </>
              )}
            </motion.div>
          )}

          {/* HISTORY VIEW */}
          {view === "history" && (
            <motion.div key="history" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              className="p-5">
              {histLoading ? (
                <div className="flex items-center justify-center h-48">
                  <div className="flex flex-col items-center gap-3">
                    <div className="w-7 h-7 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" />
                    <span className="text-sm text-[var(--text-muted)]">Loading match history...</span>
                  </div>
                </div>
              ) : history.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-48 gap-2">
                  <Clock size={28} className="text-[var(--text-muted)]" />
                  <p className="text-[var(--text-secondary)]">No match history found</p>
                </div>
              ) : (
                <>
                  <HistorySummaryBar entries={history} />
                  <div className="space-y-1.5">
                    {history.map((e, i) => <HistoryRow key={e.match_id} entry={e} index={i} />)}
                  </div>
                  <p className="text-center text-[10px] text-[var(--text-muted)] mt-4">
                    Last {history.length} matches · All queues
                  </p>
                </>
              )}
            </motion.div>
          )}

        </AnimatePresence>
      </div>
    </div>
  );
}
