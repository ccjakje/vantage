use anyhow::{anyhow, Result};
use reqwest::Client;
use regex::Regex;
use std::collections::HashMap;
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
    season_cache: HashMap<String, String>, // season_id -> "Ep X Act Y"
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

impl LocalClient {
    pub async fn new(lockfile: Lockfile, fallback_region: Option<(String, String)>) -> Result<Self> {
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

        let bearer_token     = token_resp["accessToken"].as_str().unwrap_or("").to_string();
        let entitlement_token = token_resp["token"].as_str().unwrap_or("").to_string();
        let puuid            = token_resp["subject"].as_str().unwrap_or("").to_string();

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

        let default_fb = ("eu".to_string(), "eu".to_string());
        let fb = fallback_region.unwrap_or(default_fb);
        let (region, shard) = parse_region_shard_from_log()
            .unwrap_or(fb);

        let glz_url = format!("https://glz-{}-1.{}.a.pvp.net", region, shard);
        let pd_url  = format!("https://pd.{}.a.pvp.net", shard);

        let mut instance = Self {
            client, lockfile, puuid, bearer_token, entitlement_token,
            client_version, glz_url, pd_url,
            season_cache: HashMap::new(),
        };

        // Fetch season names once at startup
        instance.preload_seasons().await;

        Ok(instance)
    }

    async fn preload_seasons(&mut self) {
        let Ok(data): Result<serde_json::Value> = self.get_pd("/content/v1/contents").await else {
            return;
        };
        let Some(seasons) = data["Seasons"].as_array() else { return; };

        // Build episode/act map
        let mut episodes: HashMap<String, String> = HashMap::new();
        let mut acts: Vec<(String, String, String)> = Vec::new(); // (id, name, parent_id)

        for s in seasons {
            let id   = s["ID"].as_str().unwrap_or("").to_string();
            let name = s["Name"].as_str().unwrap_or("").to_string();
            let stype = s["Type"].as_str().unwrap_or("");
            match stype {
                "episode" => { episodes.insert(id, name); }
                "act"     => {
                    let parent = s["ParentID"].as_str().unwrap_or("").to_string();
                    acts.push((id, name, parent));
                }
                _ => {}
            }
        }

        for (id, name, parent) in acts {
            let ep_name = episodes.get(&parent).cloned().unwrap_or_default();
            let label = if ep_name.is_empty() {
                name.clone()
            } else {
                format!("{} — {}", ep_name, name)
            };
            self.season_cache.insert(id, label);
        }
    }

    fn auth_headers(&self) -> [(&'static str, String); 4] {
        [
            ("Authorization",           format!("Bearer {}", self.bearer_token)),
            ("X-Riot-Entitlements-JWT", self.entitlement_token.clone()),
            ("X-Riot-ClientPlatform",   CLIENT_PLATFORM.to_string()),
            ("X-Riot-ClientVersion",    self.client_version.clone()),
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

    // Returns (rank, wins, total, hs%, level, peak_rank, peak_act)
    pub async fn get_player_stats(&self, puuid: &str, wr_games: u32, hs_games: u32) -> Result<(u32, u32, u32, f32, u32, u32, String)> {
        let end_index = wr_games.max(1).min(20);
        let response: serde_json::Value = self.get_pd(
            &format!("/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex={}&queue=competitive", puuid, end_index)
        ).await?;

        let matches = match response["Matches"].as_array() {
            Some(m) if !m.is_empty() => m.clone(),
            _ => return Ok((0, 0, 0, 0.0, 0, 0, String::new())),
        };

        let rank = matches.first()
            .and_then(|m| m["TierAfterUpdate"].as_u64())
            .unwrap_or(0) as u32;

        let total = matches.len() as u32;
        let wins  = matches.iter()
            .filter(|m| m["RankedRatingEarned"].as_i64().unwrap_or(-999) > 0)
            .count() as u32;

        // Peak rank from fetched games
        let peak_entry = matches.iter()
            .max_by_key(|m| m["TierAfterUpdate"].as_u64().unwrap_or(0));
        let peak_rank = peak_entry
            .and_then(|m| m["TierAfterUpdate"].as_u64())
            .unwrap_or(0) as u32;
        let peak_season_id = peak_entry
            .and_then(|m| m["SeasonID"].as_str())
            .unwrap_or("").to_string();

        // Look up from cache — O(1), no extra request
        let peak_act = self.season_cache
            .get(&peak_season_id)
            .cloned()
            .unwrap_or_default();

        // HS% from last N matches in parallel
        let hs_count = (hs_games as usize).min(10);
        let match_ids: Vec<String> = matches.iter().take(hs_count)
            .filter_map(|m| m["MatchID"].as_str().map(|s| s.to_string()))
            .collect();
        let hs = self.fetch_hs_percent(puuid, &match_ids).await.unwrap_or(0.0);

        // Account level — try account-xp, fallback to mmr identity
        let level = self.fetch_account_level(puuid).await;

        Ok((rank, wins, total, hs, level, peak_rank, peak_act))
    }

    async fn fetch_account_level(&self, puuid: &str) -> u32 {
        // Primary: account-xp endpoint
        if let Ok(resp) = self.get_pd::<serde_json::Value>(
            &format!("/account-xp/v1/players/{}", puuid)
        ).await {
            if let Some(lvl) = resp["Progress"]["Level"].as_u64() {
                return lvl as u32;
            }
        }
        // Fallback: MMR endpoint has AccountLevel in some responses
        if let Ok(resp) = self.get_pd::<serde_json::Value>(
            &format!("/mmr/v1/players/{}", puuid)
        ).await {
            if let Some(lvl) = resp["AccountLevel"].as_u64() {
                return lvl as u32;
            }
        }
        0
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
            let Some(rounds) = data["roundResults"].as_array() else { continue };
            for round in rounds {
                let Some(player_stats) = round["playerStats"].as_array() else { continue };
                for ps in player_stats {
                    if ps["subject"].as_str() != Some(puuid) { continue; }
                    let Some(damage) = ps["damage"].as_array() else { continue };
                    for dmg in damage {
                        let h = dmg["headshots"].as_u64().unwrap_or(0) as u32;
                        let b = dmg["bodyshots"].as_u64().unwrap_or(0) as u32;
                        let l = dmg["legshots"].as_u64().unwrap_or(0) as u32;
                        head_hits  += h;
                        total_hits += h + b + l;
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

    /// Fetch the most recent match summary for post-game display
    pub async fn get_last_match_summary(&self) -> Result<crate::models::MatchSummary> {
        // Try each queue type — first one that returns a match wins
        let queues = ["competitive", "unrated", "swiftplay", "spikerush", "deathmatch", ""];
        for queue in queues {
            let url = if queue.is_empty() {
                format!("/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex=1", self.puuid)
            } else {
                format!("/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex=1&queue={}", self.puuid, queue)
            };
            if let Ok(resp) = self.get_pd::<serde_json::Value>(&url).await {
                if let Some(match_id) = resp["Matches"].as_array()
                    .and_then(|m| m.first())
                    .and_then(|m| m["MatchID"].as_str())
                {
                    return self.get_match_summary(match_id).await;
                }
            }
        }
        Err(anyhow!("No recent matches found"))
    }

    /// Fetch and parse a full match by ID
    pub async fn get_match_summary(&self, match_id: &str) -> Result<crate::models::MatchSummary> {
        let data: serde_json::Value = self.get_pd(
            &format!("/match-details/v1/matches/{}", match_id)
        ).await?;

        // Parse map name: "/Game/Maps/Ascent/Ascent" → "Ascent"
        let map_raw = data["matchInfo"]["mapId"].as_str().unwrap_or("Unknown");
        let map = map_raw.rsplit('/').next().unwrap_or(map_raw).to_string();

        let mode = data["matchInfo"]["queueID"].as_str().unwrap_or("Unknown").to_string();
        let duration_ms = data["matchInfo"]["gameLengthMillis"].as_u64().unwrap_or(0);
        let duration_minutes = (duration_ms / 60000) as u32;

        // Get player names
        let puuids: Vec<String> = data["players"].as_array()
            .map(|arr| arr.iter().filter_map(|p| p["subject"].as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let names = self.get_player_names(&puuids).await.unwrap_or_default();

        // Build per-player stats
        let players_json = data["players"].as_array()
            .ok_or_else(|| anyhow!("No players in match data"))?;

        // Pre-compute HS% and damage from roundResults
        let rounds = data["roundResults"].as_array();
        let mut player_hs: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new(); // puuid -> (head, total)
        let mut player_dmg: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut player_fb: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        if let Some(rounds) = rounds {
            for round in rounds {
                let Some(player_stats) = round["playerStats"].as_array() else { continue };

                // Find first kill of the round for first blood detection
                let mut first_kill_time = u64::MAX;
                let mut first_killer = String::new();

                for ps in player_stats {
                    let puuid = ps["subject"].as_str().unwrap_or("").to_string();

                    // Damage stats
                    if let Some(damage) = ps["damage"].as_array() {
                        for dmg in damage {
                            let h = dmg["headshots"].as_u64().unwrap_or(0) as u32;
                            let b = dmg["bodyshots"].as_u64().unwrap_or(0) as u32;
                            let l = dmg["legshots"].as_u64().unwrap_or(0) as u32;
                            let d = dmg["damage"].as_u64().unwrap_or(0) as u32;
                            let entry = player_hs.entry(puuid.clone()).or_insert((0, 0));
                            entry.0 += h;
                            entry.1 += h + b + l;
                            *player_dmg.entry(puuid.clone()).or_insert(0) += d;
                        }
                    }

                    // First blood detection: find earliest kill in round
                    if let Some(kills) = ps["kills"].as_array() {
                        for kill in kills {
                            let round_time = kill["roundTime"].as_u64().unwrap_or(u64::MAX);
                            if round_time < first_kill_time {
                                first_kill_time = round_time;
                                first_killer = puuid.clone();
                            }
                        }
                    }
                }

                // Award first blood
                if !first_killer.is_empty() {
                    *player_fb.entry(first_killer).or_insert(0) += 1;
                }
            }
        }

        // Build player stats
        let mut all_players = Vec::new();
        for p in players_json {
            let puuid = p["subject"].as_str().unwrap_or("").to_string();
            let character_id = p["characterId"].as_str().unwrap_or("");
            let agent = crate::analysis::agent_from_uuid(character_id);
            let team = p["teamId"].as_str().unwrap_or("Unknown").to_string();

            let (name, tag) = names.iter()
                .find(|(id, _, _)| id == &puuid)
                .map(|(_, n, t)| (n.clone(), t.clone()))
                .unwrap_or_else(|| ("Unknown".to_string(), "????".to_string()));

            let stats = &p["stats"];
            let kills   = stats["kills"].as_u64().unwrap_or(0) as u32;
            let deaths  = stats["deaths"].as_u64().unwrap_or(0) as u32;
            let assists = stats["assists"].as_u64().unwrap_or(0) as u32;
            let score   = stats["score"].as_u64().unwrap_or(0) as u32;

            let (head, total_hits) = player_hs.get(&puuid).copied().unwrap_or((0, 0));
            let hs_percent = if total_hits > 0 { head as f32 / total_hits as f32 * 100.0 } else { 0.0 };
            let damage_dealt = player_dmg.get(&puuid).copied().unwrap_or(0);
            let first_bloods = player_fb.get(&puuid).copied().unwrap_or(0);

            all_players.push(crate::models::PlayerMatchStats {
                puuid, name, tag, agent, team,
                kills, deaths, assists, score,
                hs_percent, damage_dealt, first_bloods,
            });
        }

        // Determine my team and who won
        let my_team = all_players.iter()
            .find(|p| p.puuid == self.puuid)
            .map(|p| p.team.clone())
            .unwrap_or_else(|| "Blue".to_string());

        // Count rounds won per team
        let mut blue_rounds = 0u32;
        let mut red_rounds = 0u32;
        if let Some(rounds) = data["roundResults"].as_array() {
            for round in rounds {
                match round["winningTeam"].as_str() {
                    Some("Blue") => blue_rounds += 1,
                    Some("Red")  => red_rounds += 1,
                    _ => {}
                }
            }
        }

        let (rounds_won, rounds_lost) = if my_team == "Blue" {
            (blue_rounds, red_rounds)
        } else {
            (red_rounds, blue_rounds)
        };
        let won = rounds_won > rounds_lost;

        // Calculate ACS (Average Combat Score) = score / rounds_played
        let total_rounds = (blue_rounds + red_rounds).max(1);
        for player in &mut all_players {
            player.score = player.score / total_rounds;
        }

        let my_stats = all_players.iter()
            .find(|p| p.puuid == self.puuid)
            .cloned()
            .unwrap_or_else(|| crate::models::PlayerMatchStats {
                puuid: self.puuid.clone(),
                name: "???".to_string(),
                tag: "????".to_string(),
                agent: String::new(),
                team: my_team.clone(),
                kills: 0, deaths: 0, assists: 0, score: 0,
                hs_percent: 0.0, damage_dealt: 0, first_bloods: 0,
            });

        Ok(crate::models::MatchSummary {
            match_id: match_id.to_string(),
            map, mode, won,
            rounds_won, rounds_lost,
            duration_minutes,
            my_stats,
            all_players,
        })
    }

    /// Fetch last N matches from history — works regardless of whether tracker was running.
    /// Fetch last N matches from history.
    /// Uses /match-history/v1/history which returns ALL queues regardless
    /// of whether the tracker was running during the match.
    pub async fn get_match_history(&self, count: usize) -> Result<Vec<crate::models::MatchHistoryEntry>> {
        let count = count.min(20);

        // This endpoint returns every match ever played, sorted newest first
        let resp: serde_json::Value = self.get_pd(
            &format!("/match-history/v1/history/{}?startIndex=0&endIndex={}", self.puuid, count)
        ).await?;

        let match_ids: Vec<String> = resp["History"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["MatchID"].as_str().map(|s| s.to_string()))
            .collect();

        if match_ids.is_empty() {
            return Ok(vec![]);
        }

        // Fetch all match details in parallel
        let futs: Vec<_> = match_ids.iter()
            .map(|id| self.get_match_history_entry(id))
            .collect();
        let results = futures::future::join_all(futs).await;

        let mut entries: Vec<crate::models::MatchHistoryEntry> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        entries.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        Ok(entries)
    }

    async fn get_match_history_entry(&self, match_id: &str) -> Result<crate::models::MatchHistoryEntry> {
        let data: serde_json::Value = self.get_pd(
            &format!("/match-details/v1/matches/{}", match_id)
        ).await?;

        let map_raw = data["matchInfo"]["mapId"].as_str().unwrap_or("Unknown");
        let map = map_raw.rsplit('/').next().unwrap_or(map_raw).to_string();
        let mode = data["matchInfo"]["queueID"].as_str().unwrap_or("unknown").to_string();
        let start_time = data["matchInfo"]["gameStartMillis"].as_u64().unwrap_or(0);
        let duration_ms = data["matchInfo"]["gameLengthMillis"].as_u64().unwrap_or(0);

        // Find me in players
        let players = data["players"].as_array()
            .ok_or_else(|| anyhow!("No players"))?;
        let me = players.iter()
            .find(|p| p["subject"].as_str() == Some(&self.puuid))
            .ok_or_else(|| anyhow!("Self not found in match"))?;

        let agent = crate::analysis::agent_from_uuid(
            me["characterId"].as_str().unwrap_or("")
        );
        let my_team = me["teamId"].as_str().unwrap_or("Blue").to_string();
        let stats = &me["stats"];
        let kills   = stats["kills"].as_u64().unwrap_or(0) as u32;
        let deaths  = stats["deaths"].as_u64().unwrap_or(0) as u32;
        let assists = stats["assists"].as_u64().unwrap_or(0) as u32;
        let score   = stats["score"].as_u64().unwrap_or(0) as u32;

        // Count rounds per team
        let mut blue_rounds = 0u32;
        let mut red_rounds = 0u32;
        if let Some(rounds) = data["roundResults"].as_array() {
            for r in rounds {
                match r["winningTeam"].as_str() {
                    Some("Blue") => blue_rounds += 1,
                    Some("Red")  => red_rounds += 1,
                    _ => {}
                }
            }
        }
        let total_rounds = (blue_rounds + red_rounds).max(1);
        let (rounds_won, rounds_lost) = if my_team == "Blue" {
            (blue_rounds, red_rounds)
        } else {
            (red_rounds, blue_rounds)
        };
        let won = rounds_won > rounds_lost;
        let acs = score / total_rounds;

        // HS% for me
        let mut head = 0u32;
        let mut total_hits = 0u32;
        if let Some(rounds) = data["roundResults"].as_array() {
            for r in rounds {
                if let Some(pstats) = r["playerStats"].as_array() {
                    for ps in pstats {
                        if ps["subject"].as_str() != Some(&self.puuid) { continue; }
                        if let Some(dmg_arr) = ps["damage"].as_array() {
                            for d in dmg_arr {
                                let h = d["headshots"].as_u64().unwrap_or(0) as u32;
                                let b = d["bodyshots"].as_u64().unwrap_or(0) as u32;
                                let l = d["legshots"].as_u64().unwrap_or(0) as u32;
                                head += h;
                                total_hits += h + b + l;
                            }
                        }
                    }
                }
            }
        }
        let hs_percent = if total_hits > 0 { head as f32 / total_hits as f32 * 100.0 } else { 0.0 };

        // Rank after match (from competitive updates cache if available)
        let rank_after = me["competitiveTier"].as_u64().unwrap_or(0) as u32;

        Ok(crate::models::MatchHistoryEntry {
            match_id: match_id.to_string(),
            map,
            mode,
            won,
            rounds_won,
            rounds_lost,
            kills,
            deaths,
            assists,
            acs,
            hs_percent,
            agent,
            rank_after,
            start_time,
            duration_minutes: (duration_ms / 60000) as u32,
        })
    }
}
