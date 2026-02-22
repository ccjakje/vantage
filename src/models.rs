use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::lockfile::Lockfile;

pub struct LocalClient {
    client: Client,
    lockfile: Lockfile,
}

// --- Structs pro parsovani odpovedi ---

#[derive(Debug, Deserialize)]
pub struct Presence {
    pub puuid: String,
    pub game_name: String,
    pub game_tag: String,
}

#[derive(Debug, Deserialize)]
pub struct PresencesResponse {
    pub presences: Vec<Presence>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerMMR {
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "QueueSkills")]
    pub queue_skills: HashMap<String, QueueSkill>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueueSkill {
    #[serde(rename = "SeasonalInfoBySeasonID")]
    pub seasonal_info: Option<HashMap<String, SeasonInfo>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SeasonInfo {
    #[serde(rename = "Rank")]
    pub rank: u32,
    #[serde(rename = "NumberOfWins")]
    pub wins: u32,
}

#[derive(Debug, Deserialize)]
pub struct PregameMatch {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "AllyTeam")]
    pub ally_team: AllyTeam,
}

#[derive(Debug, Deserialize)]
pub struct AllyTeam {
    #[serde(rename = "Players")]
    pub players: Vec<PregamePlayer>,
}

#[derive(Debug, Deserialize)]
pub struct PregamePlayer {
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "CharacterID")]
    pub character_id: String,
    #[serde(rename = "PlayerIdentity")]
    pub identity: PlayerIdentity,
}

#[derive(Debug, Deserialize)]
pub struct PlayerIdentity {
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "AccountLevel")]
    pub account_level: u32,
    #[serde(rename = "Incognito")]
    pub incognito: bool,
}

#[derive(Debug, Deserialize)]
pub struct CoreGameMatch {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    #[serde(rename = "Players")]
    pub players: Vec<CoreGamePlayer>,
}

#[derive(Debug, Deserialize)]
pub struct CoreGamePlayer {
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "TeamID")]
    pub team_id: String,
    #[serde(rename = "CharacterID")]
    pub character_id: String,
    #[serde(rename = "PlayerIdentity")]
    pub identity: PlayerIdentity,
}
