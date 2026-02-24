use serde::Deserialize;


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct PregamePlayer {
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "CharacterID")]
    pub character_id: String,
    #[serde(rename = "PlayerIdentity")]
    pub identity: PlayerIdentity,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

// --- Post-game stats ---

#[derive(Debug, Clone)]
pub struct MatchSummary {
    pub match_id: String,
    pub map: String,
    pub mode: String,
    pub won: bool,
    pub rounds_won: u32,
    pub rounds_lost: u32,
    pub duration_minutes: u32,
    pub my_stats: PlayerMatchStats,
    pub all_players: Vec<PlayerMatchStats>,
}

#[derive(Debug, Clone)]
pub struct PlayerMatchStats {
    pub puuid: String,
    pub name: String,
    pub tag: String,
    pub agent: String,
    pub team: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub score: u32,
    pub hs_percent: f32,
    pub damage_dealt: u32,
    pub first_bloods: u32,
}

// --- Match history (per-match lightweight entry) ---

#[derive(Debug, Clone)]
pub struct MatchHistoryEntry {
    pub match_id: String,
    pub map: String,
    pub mode: String,
    pub won: bool,
    pub rounds_won: u32,
    pub rounds_lost: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub acs: u32,
    pub hs_percent: f32,
    pub agent: String,
    pub rank_after: u32,
    pub start_time: u64,       // Unix ms
    pub duration_minutes: u32,
}
