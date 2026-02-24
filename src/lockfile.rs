use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Lockfile {
    pub name: String,
    pub pid: u32,
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

impl Lockfile {
    pub fn base_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.protocol, self.port)
    }
}

pub fn get_lockfile_path() -> Result<PathBuf> {
    let local_app_data = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not find AppData/Local"))?;

    let path = local_app_data
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");

    Ok(path)
}

pub fn read_lockfile() -> Result<Lockfile> {
    let path = get_lockfile_path()?;

    if !path.exists() {
        return Err(anyhow!(
            "Lockfile not found — start Valorant and try again"
        ));
    }

    let content = std::fs::read_to_string(&path)?;
    let parts: Vec<&str> = content.split(':').collect();

    if parts.len() != 5 {
        return Err(anyhow!("Lockfile has unexpected format"));
    }

    Ok(Lockfile {
        name: parts[0].to_string(),
        pid: parts[1].parse()?,
        port: parts[2].parse()?,
        password: parts[3].to_string(),
        protocol: parts[4].trim().to_string(),
    })
}
