<div align="center">

<img src="src-tauri/icons/icon.png" width="80" alt="Vantage" />

# VANTAGE

**Minimal Valorant match tracker. No API key. No cloud. No bullshit.**

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?style=flat-square&logo=tauri)
![Platform](https://img.shields.io/badge/Windows-only-lightgrey?style=flat-square&logo=windows)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

</div>

---

## What it does

Vantage reads Valorant's local API — the same one the game uses internally — to give you real-time player stats before and during a match.

No Riot API key required. No data leaves your machine.

```
Agent Select → see all 5 teammates ranked, winrate, HS%, smurf flags
In-game     → same view with enemy team added
Post-game   → your ACS, KDA, HS%, first bloods vs entire lobby
History     → last 20 matches, all queues, no tracker required
```

---

## Features

- **Live match data** — rank, winrate, HS%, account level, peak rank per player
- **Smurf detection** — flags suspicious accounts based on level, rank delta, and stats
- **Match history** — fetches past matches directly from Riot's local API, works even if Vantage wasn't running
- **Agent & rank icons** — full icon set for all 28 agents and every rank
- **Zero telemetry** — no accounts, no analytics, no external requests
- **Lightweight** — Rust backend, ~5MB binary

---

## Stack

| Layer | Tech |
|---|---|
| Backend | Rust + Tokio (async) |
| GUI | Tauri v2 + React + TypeScript |
| Styling | Tailwind CSS v4 |
| Local API | Valorant Client API (localhost) |

---

## How it works

When Valorant runs, it starts a local HTTP server on a random port and writes credentials to a lockfile at:

```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
```

Vantage reads this file to authenticate with the local API — the same way Riot's own overlay tools work. No tokens are stored or transmitted.

---

## Getting started

> Requires Windows and a running Valorant client.

```bash
git clone https://github.com/ccjakje/vantage
cd vantage

# Run CLI
cargo run

# Run GUI (requires Node 18+)
npm install
npm run tauri dev
```

---

## Configuration

Config is auto-created at `%APPDATA%\vantage\config.toml` on first run.

```toml
[display]
columns = ["rank", "winrate", "hs_percent", "peak_rank", "level", "agent", "smurf"]

[match]
winrate_games = 20   # how many games to average winrate over
hs_games = 5         # how many games to average HS% over

[network]
region = "eu"
shard  = "eu"
```

---

## Privacy & security

- Reads **only** from localhost — no outbound connections except to `auth.riotgames.com` for name resolution
- Lockfile credentials are used in-memory only, never written or logged
- No account creation, no cloud sync, nothing stored outside your machine

---

## Disclaimer

Vantage is not affiliated with or endorsed by Riot Games. Use at your own risk. Reading the local API is a grey area — Vantage does not modify game memory or inject into any process.

---

<div align="center">
<sub>Built with Rust + Tauri · MIT License</sub>
</div>
