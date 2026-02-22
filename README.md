
---

# 🎯 Vantage – Valorant Local API Tracker

A lightweight Valorant tracker built on top of the official local client API.
Tracks match data, player stats, and live session information directly from the running game client.

---

## 🚀 Features

* 📡 Connects to Valorant local client API
* 📊 Match tracking
* 🧠 Player performance stats
* 🗂 Local data processing
* ⚡ Fast and lightweight (written in Rust)

---

## 🛠 Tech Stack

* **Rust**
* Valorant Local Client API
* JSON parsing & HTTP client

---

## 📦 Installation

### 1️⃣ Clone repository

```bash
git clone https://github.com/yourname/vantage.git
cd vantage
```

### 2️⃣ Build project

```bash
cargo build --release
```

### 3️⃣ Run

```bash
cargo run
```

---

## 🔌 How It Works

Valorant exposes a local API endpoint while the client is running.

This project:

1. Authenticates against the local client
2. Fetches live data
3. Parses match & player information
4. Displays or processes it

---

## 📁 Project Structure

```
src/
 ├── main.rs        # Entry point
 ├── local_api.rs   # Local client communication
 ├── analysis.rs    # Data processing logic
 ├── display.rs     # Output formatting
 ├── models.rs      # Data structures
```

---

## ⚠️ Requirements

* Valorant must be running
* Local API must be accessible
* Windows (for now)

---

## 📌 Future Improvements

* [ ] Live match overlay
* [ ] Web dashboard
* [ ] Match history export
* [ ] Player performance analytics
* [ ] GUI version

---

## 🧠 Why This Project?

* Many existing Valorant trackers rely on heavy third-party platforms, background services, and ad-based monetization models.

* Vantage was created as a lightweight, independent alternative focused purely on performance, transparency, and efficiency.

* The goal is simple:
    Provide useful match insights without unnecessary overhead.

---

## 📜 Disclaimer

This tool uses the official local client API and does not interact with the game in unauthorized ways.
Use responsibly and at your own risk.

---

## 🤝 Contributing

Pull requests are welcome.
If you have ideas for improvements, feel free to open an issue.

---

## ⭐ License

MIT (or whatever you choose)

---
