use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VantageConfig {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub overlay: OverlayConfig,
    #[serde(rename = "match")]
    #[serde(default)]
    pub match_config: MatchConfig,
    #[serde(default)]
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Columns displayed for each player, in this order
    #[serde(default = "default_columns")]
    pub columns: Vec<String>,
}

fn default_columns() -> Vec<String> {
    vec![
        "rank".into(), "winrate".into(), "hs_percent".into(),
        "peak_rank".into(), "level".into(), "agent".into(), "smurf".into(),
    ]
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self { columns: default_columns() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    #[serde(default = "default_keybind")]
    pub keybind: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_keybind() -> String { "F1".into() }
fn default_opacity() -> f32 { 0.9 }
fn default_true() -> bool { true }

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            keybind: default_keybind(),
            opacity: default_opacity(),
            enabled: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    /// How many recent games to use for WR calculation (max 20)
    #[serde(default = "default_wr_games")]
    pub winrate_games: u32,
    /// How many recent games to use for HS% calculation (max 10)
    #[serde(default = "default_hs_games")]
    pub hs_games: u32,
}

fn default_wr_games() -> u32 { 20 }
fn default_hs_games() -> u32 { 5 }

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            winrate_games: default_wr_games(),
            hs_games: default_hs_games(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Fallback region if detection from ShooterGame.log fails
    #[serde(default = "default_region")]
    pub region: String,
    /// Fallback shard
    #[serde(default = "default_shard")]
    pub shard: String,
}

fn default_region() -> String { "eu".into() }
fn default_shard() -> String { "eu".into() }

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            region: default_region(),
            shard: default_shard(),
        }
    }
}

impl Default for VantageConfig {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            overlay: OverlayConfig::default(),
            match_config: MatchConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl VantageConfig {
    /// Returns the config directory: %APPDATA%\vantage
    fn config_dir() -> Result<PathBuf> {
        let app_data = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find APPDATA directory"))?;
        Ok(app_data.join("vantage"))
    }

    /// Returns the config file path: %APPDATA%\vantage\config.toml
    fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load config from disk. If the file doesn't exist, create it with defaults.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut config: VantageConfig = toml::from_str(&content)
            .unwrap_or_else(|e| {
                eprintln!("  [!] Config parse error: {} — using defaults", e);
                VantageConfig::default()
            });

        // Clamp values to valid ranges
        config.match_config.winrate_games = config.match_config.winrate_games.min(20);
        config.match_config.hs_games = config.match_config.hs_games.min(10);

        Ok(config)
    }

    /// Save current config to disk
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
