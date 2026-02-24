# Vantage

> A powerful Valorant tracker with CLI, GUI, and live in-game overlays.  
> Uses Valorant's local API — no API key required, works on private profiles.

---

## Features

| Feature | Status |
|---|---|
| CLI interface | ✅ Done |
| Live match detection (watch mode) | ✅ Done |
| Player ranks (incl. private profiles) | ✅ Done |
| Win rate (last 20 games) | ✅ Done |
| Headshot % (last 5 games) | ✅ Done |
| Peak rank + act/episode | ✅ Done |
| Account level | ✅ Done |
| Smurf detection | ✅ Done |
| Agent display | ✅ Done |
| Average team rank | ✅ Done |
| GUI app (Tauri) | 🔲 Planned |
| Agent Select overlay | 🔲 Planned |
| In-game stats overlay (keybind) | 🔲 Planned |
| Tab overlay (per-round stats) | 🔲 Planned |
| Post-game summary | 🔲 Planned |
| Configurable columns (CLI + GUI) | 🔲 Planned |

---

## Roadmap

### Phase 1 — CLI (current) ✅
Core data pipeline working via terminal.  
All stats fetched from Valorant's local API — no external API key needed.

### Phase 2 — Tauri GUI App
A standalone desktop app with a proper UI.  
Settings panel to configure which columns are shown in overlays.  
Launches minimized to tray, activates automatically when Valorant is detected.

### Phase 3 — Agent Select Overlay
A small transparent window that appears automatically when agent selection starts.  
Shows your team's ranks, WR, HS%, and smurf flags.  
Auto-dismisses when the match begins.

### Phase 4 — In-Game Stats Overlay
A toggleable overlay activated by a configurable keybind (default: `F1`).  
Shows both teams with full stats — rank, WR, HS%, peak rank, smurf detection.  
Designed to be used in **Borderless Windowed** mode.

### Phase 5 — Tab Overlay (Per-Round Stats)
Detects when the player presses Tab in-game (scoreboard key).  
Renders a panel below the native scoreboard showing per-round breakdown:
- Kills per round
- Headshot %
- Damage dealt
- Additional user-configured columns (via `config.toml` or GUI settings)

User can choose which columns to display and in what order.

### Phase 6 — Post-Game Summary
After the match ends, shows a detailed summary:
- ACS (Average Combat Score)
- KAST %
- Kill / Death / Assist
- Headshot %
- MVP highlight
- Match result + round score

---

## Overlay Architecture

All overlays are **Tauri windows** sharing the same Rust backend.

```
vantage (Tauri app)
├── main window          → GUI app, settings, match history
├── agent-select window  → auto-shows during agent select, auto-hides
├── ingame window        → toggled by keybind (F1), shows both teams
└── tab window           → appears when Tab is held, per-round stats
```

Each window is:
- `transparent: true` — no background
- `always_on_top: true` — renders over Valorant
- `decorations: false` — no titlebar/borders
- `skip_taskbar: true` — hidden from taskbar

The Rust backend polls the local API every 2 seconds and emits events to all windows.

---

## Configuration

Settings are stored in `config.toml` (auto-created on first run).

```toml
[overlay]
keybind = "F1"           # Toggle in-game overlay
tab_overlay = true       # Enable tab overlay

[columns]
# Columns shown in overlays and CLI — order matters
show = ["rank", "winrate", "hs_percent", "peak_rank", "level", "smurf"]
# Available: rank, winrate, hs_percent, peak_rank, peak_act, level, agent, smurf

[tab_overlay]
# Columns shown in per-round stats panel
show = ["kills", "hs_percent", "damage", "assists"]
# Available: kills, deaths, hs_percent, damage, assists, economy, first_blood

[appearance]
theme = "dark"           # dark / light
opacity = 0.9
```

---

## Project Structure

```
vantage/
├── src/
│   ├── main.rs          # Entry point, CLI commands, watch loop
│   ├── lockfile.rs      # Reads Valorant lockfile (port + credentials)
│   ├── local_api.rs     # All API calls (local + GLZ + PD servers)
│   ├── models.rs        # Data structures (Serde deserialization)
│   ├── display.rs       # Terminal output, colors, formatting
│   └── analysis.rs      # Smurf detection, agent UUID mapping
├── Cargo.toml
└── README.md
```

> When Tauri is added, the structure will expand with `src-tauri/` and `src/` (frontend).

---

## How It Works

Valorant exposes a local REST API on `127.0.0.1` when running.  
Access credentials are stored in a lockfile:

```
%LocalAppData%\Riot Games\Riot Client\Config\lockfile
```

From the lockfile we read the **port** and **password** used for Basic Auth.  
We then fetch an **entitlement token** and **bearer token** from the local API.  
These tokens are used to call Riot's remote servers (GLZ and PD) for match and MMR data.

### API Servers

| Server | Used for |
|---|---|
| `127.0.0.1:{port}` | Auth tokens, session info |
| `glz-{region}-1.{shard}.a.pvp.net` | Live match data (pregame, coregame) |
| `pd.{shard}.a.pvp.net` | Player data (MMR, match history, names) |

### Why private profiles work

Private profiles only block **Riot's public API**.  
The local Valorant client fetches rank data for all 10 players in a lobby internally.  
We read this data from the same local API — which has no privacy restrictions.

---

## Usage

### Prerequisites
- Rust + Cargo: https://rustup.rs
- Visual Studio Build Tools 2022+ (Desktop development with C++)

### Build

```bash
git clone https://github.com/yourname/vantage
cd vantage
cargo build --release
```

Binary: `target/release/vantage.exe`

### CLI Usage

```bash
# Single check
vantage

# Watch mode — auto-detects matches
vantage --watch
vantage -w
```

---

## Adding New Agents

Agent UUIDs are mapped in `src/analysis.rs` → `agent_from_uuid()`.  
New agents will show as blank until the map is updated.

Find UUIDs at: https://valorant-api.com/v1/agents

```rust
// Add to the match statement in analysis.rs:
"UUID-HERE" => "AgentName",
```

---

## Smurf Detection

| Signal | Points |
|---|---|
| Account level < 50 | +30 |
| Headshot % ≥ 30% | +25 |
| Win rate ≥ 65% (min 5 games) | +25 |
| Low rank but high HS/WR | +20 |

| Score | Label |
|---|---|
| 75–100 | ⚠ SMURF |
| 50–74 | ? Sus |
| 0–49 | (none) |

---

## Data Sources

| Data | Source | Requires key? |
|---|---|---|
| Live match players | Valorant Local API | ❌ |
| Player ranks | PD server via local tokens | ❌ |
| Private profile ranks | Local API (bypasses privacy) | ❌ |
| Win rate | PD `/competitiveupdates` | ❌ |
| Headshot % | PD `/match-details` | ❌ |
| Account level | PD `/account-xp` | ❌ |
| Player names | PD `/name-service` | ❌ |
| Peak rank | PD `/competitiveupdates` history | ❌ |
| Per-round stats | PD `/match-details` | ❌ |

---

## Legal

Not affiliated with Riot Games.  
Uses Riot's local API for personal use only — no gameplay modification or automation.  
"VALORANT" is a trademark of Riot Games.
