import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, RotateCcw } from "lucide-react";
import { motion } from "framer-motion";

interface VantageConfig {
  display: { columns: string[] };
  overlay: { keybind: string; opacity: number; enabled: boolean };
  match_config: { winrate_games: number; hs_games: number };
  network: { region: string; shard: string };
}

const ALL_COLUMNS = [
  { key: "rank", label: "Rank" },
  { key: "winrate", label: "Win Rate" },
  { key: "hs_percent", label: "HS%" },
  { key: "peak_rank", label: "Peak Rank" },
  { key: "level", label: "Level" },
  { key: "agent", label: "Agent" },
  { key: "smurf", label: "Smurf Detection" },
];

export default function Settings() {
  const [config, setConfig] = useState<VantageConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<VantageConfig>("get_config").then(setConfig).catch(console.error);
  }, []);

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await invoke("save_config", { config });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Save failed:", e);
    } finally {
      setSaving(false);
    }
  };

  const toggleColumn = (key: string) => {
    if (!config) return;
    const cols = config.display.columns.includes(key)
      ? config.display.columns.filter((c) => c !== key)
      : [...config.display.columns, key];
    setConfig({ ...config, display: { ...config.display, columns: cols } });
  };

  if (!config) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-6 h-6 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto p-8">
      <h1 className="text-xl font-bold mb-8">Settings</h1>

      {/* Display Columns */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-[var(--text-secondary)] tracking-wider uppercase mb-4">
          Display Columns
        </h2>
        <div className="space-y-2">
          {ALL_COLUMNS.map((col) => (
            <label
              key={col.key}
              className="flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)] cursor-pointer hover:border-[var(--border-light)] transition-colors"
            >
              <span className="text-sm">{col.label}</span>
              <div className="relative">
                <input
                  type="checkbox"
                  checked={config.display.columns.includes(col.key)}
                  onChange={() => toggleColumn(col.key)}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-[var(--border)] rounded-full peer-checked:bg-[var(--accent)] transition-colors" />
                <div className="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform peer-checked:translate-x-4" />
              </div>
            </label>
          ))}
        </div>
      </section>

      {/* Match Settings */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-[var(--text-secondary)] tracking-wider uppercase mb-4">
          Match Analysis
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <div>
              <span className="text-sm">Win Rate Games</span>
              <p className="text-xs text-[var(--text-muted)]">Number of recent games for WR calculation (max 20)</p>
            </div>
            <input
              type="number"
              min={1}
              max={20}
              value={config.match_config.winrate_games}
              onChange={(e) =>
                setConfig({
                  ...config,
                  match_config: { ...config.match_config, winrate_games: Math.min(20, Math.max(1, parseInt(e.target.value) || 1)) },
                })
              }
              className="w-16 h-8 text-center bg-[var(--bg-input)] border border-[var(--border)] rounded text-sm text-white focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
          <div className="flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <div>
              <span className="text-sm">HS% Games</span>
              <p className="text-xs text-[var(--text-muted)]">Number of recent games for HS% calculation (max 10)</p>
            </div>
            <input
              type="number"
              min={1}
              max={10}
              value={config.match_config.hs_games}
              onChange={(e) =>
                setConfig({
                  ...config,
                  match_config: { ...config.match_config, hs_games: Math.min(10, Math.max(1, parseInt(e.target.value) || 1)) },
                })
              }
              className="w-16 h-8 text-center bg-[var(--bg-input)] border border-[var(--border)] rounded text-sm text-white focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
        </div>
      </section>

      {/* Network */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-[var(--text-secondary)] tracking-wider uppercase mb-4">
          Network
        </h2>
        <div className="grid grid-cols-2 gap-4">
          <div className="px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <label className="text-xs text-[var(--text-muted)] block mb-1">Fallback Region</label>
            <input
              type="text"
              value={config.network.region}
              onChange={(e) =>
                setConfig({ ...config, network: { ...config.network, region: e.target.value } })
              }
              className="w-full h-8 bg-[var(--bg-input)] border border-[var(--border)] rounded px-3 text-sm text-white focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
          <div className="px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <label className="text-xs text-[var(--text-muted)] block mb-1">Fallback Shard</label>
            <input
              type="text"
              value={config.network.shard}
              onChange={(e) =>
                setConfig({ ...config, network: { ...config.network, shard: e.target.value } })
              }
              className="w-full h-8 bg-[var(--bg-input)] border border-[var(--border)] rounded px-3 text-sm text-white focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
        </div>
      </section>

      {/* Overlay */}
      <section className="mb-10">
        <h2 className="text-sm font-semibold text-[var(--text-secondary)] tracking-wider uppercase mb-4">
          Overlay
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <span className="text-sm">Keybind</span>
            <input
              type="text"
              value={config.overlay.keybind}
              onChange={(e) =>
                setConfig({ ...config, overlay: { ...config.overlay, keybind: e.target.value } })
              }
              className="w-20 h-8 text-center bg-[var(--bg-input)] border border-[var(--border)] rounded text-sm text-white focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
          <div className="flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-card)] border border-[var(--border)]">
            <span className="text-sm">Opacity</span>
            <div className="flex items-center gap-3">
              <input
                type="range"
                min={0.1}
                max={1}
                step={0.05}
                value={config.overlay.opacity}
                onChange={(e) =>
                  setConfig({ ...config, overlay: { ...config.overlay, opacity: parseFloat(e.target.value) } })
                }
                className="w-32 accent-[var(--accent)]"
              />
              <span className="text-xs text-[var(--text-muted)] w-10 text-right">
                {(config.overlay.opacity * 100).toFixed(0)}%
              </span>
            </div>
          </div>
        </div>
      </section>

      {/* Save Button */}
      <motion.button
        whileTap={{ scale: 0.97 }}
        onClick={handleSave}
        disabled={saving}
        className={`w-full h-11 rounded-lg font-medium text-sm flex items-center justify-center gap-2 transition-all ${
          saved
            ? "bg-[var(--green)] text-black"
            : "bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white"
        }`}
      >
        {saved ? (
          <>
            <RotateCcw size={14} /> Saved!
          </>
        ) : (
          <>
            <Save size={14} /> {saving ? "Saving..." : "Save Settings"}
          </>
        )}
      </motion.button>
    </div>
  );
}
