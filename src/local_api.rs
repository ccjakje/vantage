use anyhow::{anyhow, Result};
use reqwest::Client;
use regex::Regex;
use std::path::PathBuf;

use crate::lockfile::Lockfile;
use crate::models::*;

const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

pub struct LocalClient {
    client: Client,
    #[allow(dead_code)]
    lockfile: Lockfile,
    pub puuid: String,
    bearer_token: String,
    entitlement_token: String,
    client_version: String,
    glz_url: String,
    pd_url: String,
}

fn get_shootergame_log_path() -> PathBuf {
    let local = dirs::data_local_dir().unwrap_or_default();
    local.join("VALORANT").join("Saved").join("Logs").join("ShooterGame.log")
}

fn parse_region_shard_from_log() -> Result<(String, String)> {
    let log_path = get_shootergame_log_path();
    let content = std::fs::read_to_string(&log_path)
        .map_err(|_| anyhow!("Could not read ShooterGame.log"))?;
    let re = Regex::new(r"https://glz-(.+?)-1\.(.+?)\.a\.pvp\.net")?;
    if let Some(caps) = re.captures(&content) {
        return Ok((caps[1].to_string(), caps[2].to_string()));
    }
    Err(anyhow!("Could not find region/shard in ShooterGame.log"))
}

fn short_season_id(id: &str) -> String {
    // Fallback — show first 8 chars of UUID
    id.chars().take(8).collect()
}

impl LocalClient {
    pub async fn new(lockfile: Lockfile) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let basic_auth = {
            use base64::{engine::general_purpose, Engine as _};
            let creds = format!("riot:{}", lockfile.password);
            format!("Basic {}", general_purpose::STANDARD.encode(creds))
        };

        let token_resp: serde_json::Value = client
            .get(format!("{}/entitlements/v1/token", lockfile.base_url()))
            .header("Authorization", &basic_auth)
            .send().await?.json().await?;

        let bearer_token = token_resp["accessToken"].as_str().unwrap_or("").to_string();
        let entitlement_token = token_resp["token"].as_str().unwrap_or("").to_string();
        let puuid = token_resp["subject"].as_str().unwrap_or("").to_string();

        let version_resp: serde_json::Value = client
            .get(format!("{}/agent/v1/build-info", lockfile.base_url()))
            .header("Authorization", &basic_auth)
            .send().await?.json().await?;

        let client_version = format!(
            "{}-shipping-{}-{}",
            version_resp["branch"].as_str().unwrap_or("release-09.00"),
            version_resp["version"].as_str().unwrap_or("9"),
            version_resp["buildVersion"].as_str().unwrap_or("0")
        );

        let (region, shard) = parse_region_shard_from_log()
            .unwrap_or_else(|_| ("eu".to_string(), "eu".to_string()));

        let glz_url = format!("https://glz-{}-1.{}.a.pvp.net", region, shard);
        let pd_url  = format!("https://pd.{}.a.pvp.net", shard);

        Ok(Self { client, lockfile, puuid, bearer_token, entitlement_token, client_version, glz_url, pd_url })
    }

    fn auth_headers(&self) -> [(&'static str, String); 4] {
        [
            ("Authorization",            format!("Bearer {}", self.bearer_token)),
            ("X-Riot-Entitlements-JWT",  self.entitlement_token.clone()),
            ("X-Riot-ClientPlatform",    CLIENT_PLATFORM.to_string()),
            ("X-Riot-ClientVersion",     self.client_version.clone()),
        ]
    }

    async fn get_glz<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.glz_url, path);
        let mut req = self.client.get(&url);
        for (k, v) in self.auth_headers() { req = req.header(k, v); }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("API error {}: {}", resp.status(), path));
        }
        Ok(resp.json::<T>().await?)
    }

    async fn get_pd<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.pd_url, path);
        let mut req = self.client.get(&url);
        for (k, v) in self.auth_headers() { req = req.header(k, v); }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("API error {}: {}", resp.status(), path));
        }
        Ok(resp.json::<T>().await?)
    }

    pub async fn get_pregame_match_id(&self) -> Result<String> {
        let response: serde_json::Value =
            self.get_glz(&format!("/pregame/v1/players/{}", self.puuid)).await?;
        response["MatchID"].as_str().map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Not in pre-game"))
    }

    pub async fn get_pregame_match(&self, match_id: &str) -> Result<PregameMatch> {
        self.get_glz(&format!("/pregame/v1/matches/{}", match_id)).await
    }

    pub async fn get_coregame_match_id(&self) -> Result<String> {
        let response: serde_json::Value =
            self.get_glz(&format!("/core-game/v1/players/{}", self.puuid)).await?;
        response["MatchID"].as_str().map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Not in a match"))
    }

    pub async fn get_coregame_match(&self, match_id: &str) -> Result<CoreGameMatch> {
        self.get_glz(&format!("/core-game/v1/matches/{}", match_id)).await
    }

    // Returns (rank, wins, total_games, hs_percent, account_level, peak_rank, peak_act)
    pub async fn get_player_stats(&self, puuid: &str) -> Result<(u32, u32, u32, f32, u32, u32, String)> {
        let response: serde_json::Value = self.get_pd(
            &format!("/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex=100&queue=competitive", puuid)
        ).await?;

        let matches = match response["Matches"].as_array() {
            Some(m) if !m.is_empty() => m.clone(),
            _ => return Ok((0, 0, 0, 0.0, 0, 0, String::new())),
        };

        let rank = matches.first()
            .and_then(|m| m["TierAfterUpdate"].as_u64())
            .unwrap_or(0) as u32;

        let total = matches.len() as u32;
        let wins = matches.iter()
            .filter(|m| m["RankedRatingEarned"].as_i64().unwrap_or(-999) > 0)
            .count() as u32;

        // Peak rank — highest tier ever
        let peak_entry = matches.iter()
            .max_by_key(|m| m["TierAfterUpdate"].as_u64().unwrap_or(0));
        let peak_rank = peak_entry
            .and_then(|m| m["TierAfterUpdate"].as_u64())
            .unwrap_or(0) as u32;
        let peak_season_id = peak_entry
            .and_then(|m| m["SeasonID"].as_str())
            .unwrap_or("")
            .to_string();

        // Resolve season name
        let peak_act = self.resolve_season_name(&peak_season_id).await;

        // HS% from last 5 matches
        let match_ids: Vec<String> = matches.iter().take(5)
            .filter_map(|m| m["MatchID"].as_str().map(|s| s.to_string()))
            .collect();
        let hs_percent = self.fetch_hs_percent(puuid, &match_ids).await.unwrap_or(0.0);

        // Account level
        let level_resp: serde_json::Value = self.get_pd(
            &format!("/account-xp/v1/players/{}", puuid)
        ).await.unwrap_or(serde_json::Value::Null);
        let account_level = level_resp["Progress"]["Level"].as_u64().unwrap_or(0) as u32;

        Ok((rank, wins, total, hs_percent, account_level, peak_rank, peak_act))
    }

    async fn resolve_season_name(&self, season_id: &str) -> String {
        if season_id.is_empty() { return String::new(); }

        let Ok(data): Result<serde_json::Value> = self.get_pd("/content/v1/contents").await else {
            return short_season_id(season_id);
        };

        let Some(seasons) = data["Seasons"].as_array() else {
            return short_season_id(season_id);
        };

        for season in seasons {
            if season["ID"].as_str() == Some(season_id) {
                if let Some(name) = season["Name"].as_str() {
                    return name.to_string();
                }
            }
        }

        short_season_id(season_id)
    }

    async fn fetch_hs_percent(&self, puuid: &str, match_ids: &[String]) -> Result<f32> {
        let mut total_hits = 0u32;
        let mut head_hits  = 0u32;

        let paths: Vec<String> = match_ids.iter()
            .map(|id| format!("/match-details/v1/matches/{}", id))
            .collect();

        let futures: Vec<_> = paths.iter()
            .map(|path| self.get_pd::<serde_json::Value>(path))
            .collect();

        let results = futures::future::join_all(futures).await;

        for result in results {
            let Ok(data) = result else { continue };

            // PD API structure: roundResults[].playerStats[].damage[].headshots
            let Some(rounds) = data["roundResults"].as_array() else { continue };

            for round in rounds {
                let Some(player_stats) = round["playerStats"].as_array() else { continue };

                for ps in player_stats {
                    if ps["subject"].as_str() != Some(puuid) { continue; }

                    let Some(damage) = ps["damage"].as_array() else { continue };
                    for dmg in damage {
                        head_hits  += dmg["headshots"].as_u64().unwrap_or(0) as u32;
                        let body   = dmg["bodyshots"].as_u64().unwrap_or(0) as u32;
                        let legs   = dmg["legshots"].as_u64().unwrap_or(0) as u32;
                        total_hits += dmg["headshots"].as_u64().unwrap_or(0) as u32 + body + legs;
                    }
                }
            }
        }

        if total_hits == 0 { return Ok(0.0); }
        Ok(head_hits as f32 / total_hits as f32 * 100.0)
    }

    pub async fn get_player_names(&self, puuids: &[String]) -> Result<Vec<(String, String, String)>> {
        let url = format!("{}/name-service/v2/players", self.pd_url);
        let mut req = self.client.put(&url);
        for (k, v) in self.auth_headers() { req = req.header(k, v); }
        let resp = req.json(puuids).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Name service error: {}", resp.status()));
        }
        let data: Vec<serde_json::Value> = resp.json().await?;
        Ok(data.iter().map(|p| (
            p["Subject"].as_str().unwrap_or("").to_string(),
            p["GameName"].as_str().unwrap_or("Unknown").to_string(),
            p["TagLine"].as_str().unwrap_or("????").to_string(),
        )).collect())
    }
}
