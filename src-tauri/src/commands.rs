use serde::Serialize;
use tauri::State;

use crate::AppState;
use vantage::analysis;
use vantage::config::VantageConfig;
use vantage::lockfile::read_lockfile;
use vantage::local_api::LocalClient;

// ── Serializable types for the frontend ──────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PlayerRow {
    pub name: String,
    pub tag: String,
    pub agent: String,
    pub rank: u32,
    pub rank_name: String,
    pub wins: u32,
    pub total: u32,
    pub winrate: f32,
    pub hs: f32,
    pub level: u32,
    pub peak_rank: u32,
    pub peak_rank_name: String,
    pub peak_act: String,
    pub is_self: bool,
    pub smurf_score: u8,
    pub smurf_label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchData {
    pub phase: String,
    pub blue_team: Vec<PlayerRow>,
    pub red_team: Vec<PlayerRow>,
    pub blue_avg_rank: String,
    pub red_avg_rank: String,
    pub map: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostGameData {
    pub match_id: String,
    pub map: String,
    pub mode: String,
    pub won: bool,
    pub rounds_won: u32,
    pub rounds_lost: u32,
    pub duration_minutes: u32,
    pub players: Vec<PostGamePlayer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostGamePlayer {
    pub name: String,
    pub tag: String,
    pub agent: String,
    pub team: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub acs: u32,
    pub hs_percent: f32,
    pub damage: u32,
    pub first_bloods: u32,
    pub is_self: bool,
}

// ── Helper ──────────────────────────────────────────────────────

fn rank_name(tier: u32) -> String {
    vantage::display::rank_from_tier(tier).to_string()
}

async fn fetch_rows(client: &LocalClient, puuids: &[String], agents: &std::collections::HashMap<String, String>, config: &VantageConfig) -> Vec<PlayerRow> {
    let names = client.get_player_names(puuids).await.unwrap_or_default();
    let hs_games = config.match_config.hs_games;
    let wr_games = config.match_config.winrate_games;

    let futs: Vec<_> = puuids.iter()
        .map(|p| client.get_player_stats(p, wr_games, hs_games))
        .collect();
    let results = futures::future::join_all(futs).await;

    puuids.iter().enumerate().map(|(i, puuid)| {
        let (rank, wins, total, hs, level, peak_rank, peak_act) = match &results[i] {
            Ok(s) => s.clone(),
            Err(_) => (0, 0, 0, 0.0, 0, 0, String::new()),
        };
        let (name, tag) = names.iter()
            .find(|(p, _, _)| p == puuid)
            .map(|(_, n, t)| (n.clone(), t.clone()))
            .unwrap_or_else(|| ("Unknown".into(), "????".into()));
        let agent = agents.get(puuid).cloned().unwrap_or_default();
        let is_self = puuid == &client.puuid;
        let winrate = if total > 0 { wins as f32 / total as f32 * 100.0 } else { 0.0 };
        let smurf = analysis::SmurfFlags::analyze(level, rank, hs, wins, total);

        PlayerRow {
            name, tag, agent,
            rank, rank_name: rank_name(rank),
            wins, total, winrate, hs,
            level, peak_rank, peak_rank_name: rank_name(peak_rank), peak_act,
            is_self,
            smurf_score: smurf.score,
            smurf_label: smurf.label().map(|s| s.to_string()),
        }
    }).collect()
}

// ── Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn init_client(state: State<'_, AppState>) -> Result<String, String> {
    let lockfile = read_lockfile().map_err(|e| e.to_string())?;
    let cfg = state.config.lock().await;
    let fallback = (cfg.network.region.clone(), cfg.network.shard.clone());
    drop(cfg);
    let client = LocalClient::new(lockfile, Some(fallback)).await.map_err(|e| e.to_string())?;
    let puuid = client.puuid.clone();
    *state.client.lock().await = Some(client);
    Ok(puuid)
}

#[tauri::command]
pub async fn get_current_phase(state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.client.lock().await;
    let client = guard.as_ref().ok_or("Client not initialized")?;

    if client.get_pregame_match_id().await.is_ok() {
        return Ok("pregame".into());
    }
    if client.get_coregame_match_id().await.is_ok() {
        return Ok("coregame".into());
    }
    Ok("menu".into())
}

#[tauri::command]
pub async fn get_match_data(state: State<'_, AppState>) -> Result<MatchData, String> {
    let guard = state.client.lock().await;
    let client = guard.as_ref().ok_or("Client not initialized")?;
    let cfg_guard = state.config.lock().await;
    let config = cfg_guard.clone();
    drop(cfg_guard);

    // Try pregame first
    if let Ok(match_id) = client.get_pregame_match_id().await {
        let match_data = client.get_pregame_match(&match_id).await.map_err(|e| e.to_string())?;
        let mut agents = std::collections::HashMap::new();
        let puuids: Vec<String> = match_data.ally_team.players.iter().map(|p| {
            agents.insert(p.subject.clone(), analysis::agent_from_uuid(&p.character_id));
            p.subject.clone()
        }).collect();
        let rows = fetch_rows(client, &puuids, &agents, &config).await;
        let tiers: Vec<u32> = rows.iter().map(|r| r.rank).collect();
        return Ok(MatchData {
            phase: "pregame".into(),
            blue_team: rows,
            red_team: vec![],
            blue_avg_rank: vantage::display::avg_rank_label(&tiers),
            red_avg_rank: String::new(),
            map: None,
            mode: None,
        });
    }

    // Try coregame
    if let Ok(match_id) = client.get_coregame_match_id().await {
        let match_data = client.get_coregame_match(&match_id).await.map_err(|e| e.to_string())?;
        let mut agents = std::collections::HashMap::new();
        for p in &match_data.players {
            agents.insert(p.subject.clone(), analysis::agent_from_uuid(&p.character_id));
        }
        let blue_puuids: Vec<String> = match_data.players.iter().filter(|p| p.team_id == "Blue").map(|p| p.subject.clone()).collect();
        let red_puuids: Vec<String> = match_data.players.iter().filter(|p| p.team_id != "Blue").map(|p| p.subject.clone()).collect();

        let (blue_rows, red_rows) = futures::future::join(
            fetch_rows(client, &blue_puuids, &agents, &config),
            fetch_rows(client, &red_puuids, &agents, &config),
        ).await;

        let blue_tiers: Vec<u32> = blue_rows.iter().map(|r| r.rank).collect();
        let red_tiers: Vec<u32> = red_rows.iter().map(|r| r.rank).collect();

        return Ok(MatchData {
            phase: "coregame".into(),
            blue_team: blue_rows,
            red_team: red_rows,
            blue_avg_rank: vantage::display::avg_rank_label(&blue_tiers),
            red_avg_rank: vantage::display::avg_rank_label(&red_tiers),
            map: None,
            mode: None,
        });
    }

    Ok(MatchData {
        phase: "menu".into(),
        blue_team: vec![], red_team: vec![],
        blue_avg_rank: String::new(), red_avg_rank: String::new(),
        map: None, mode: None,
    })
}

#[tauri::command]
pub async fn get_post_game(state: State<'_, AppState>) -> Result<PostGameData, String> {
    let guard = state.client.lock().await;
    let client = guard.as_ref().ok_or("Client not initialized")?;
    let summary = client.get_last_match_summary().await.map_err(|e| e.to_string())?;

    let players = summary.all_players.iter().map(|p| PostGamePlayer {
        name: p.name.clone(),
        tag: p.tag.clone(),
        agent: p.agent.clone(),
        team: p.team.clone(),
        kills: p.kills,
        deaths: p.deaths,
        assists: p.assists,
        acs: p.score,
        hs_percent: p.hs_percent,
        damage: p.damage_dealt,
        first_bloods: p.first_bloods,
        is_self: p.puuid == client.puuid,
    }).collect();

    Ok(PostGameData {
        match_id: summary.match_id,
        map: summary.map,
        mode: summary.mode,
        won: summary.won,
        rounds_won: summary.rounds_won,
        rounds_lost: summary.rounds_lost,
        duration_minutes: summary.duration_minutes,
        players,
    })
}

#[derive(Debug, Clone, Serialize)]
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
    pub rank_after_name: String,
    pub start_time: u64,
    pub duration_minutes: u32,
}

#[tauri::command]
pub async fn get_match_history(count: usize, state: State<'_, AppState>) -> Result<Vec<MatchHistoryEntry>, String> {
    let guard = state.client.lock().await;
    let client = guard.as_ref().ok_or("Client not initialized")?;
    let entries = client.get_match_history(count.min(20)).await.map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(|e| {
        let rname = rank_name(e.rank_after);
        MatchHistoryEntry {
            match_id: e.match_id,
            map: e.map,
            mode: e.mode,
            won: e.won,
            rounds_won: e.rounds_won,
            rounds_lost: e.rounds_lost,
            kills: e.kills,
            deaths: e.deaths,
            assists: e.assists,
            acs: e.acs,
            hs_percent: e.hs_percent,
            agent: e.agent,
            rank_after: e.rank_after,
            rank_after_name: rname,
            start_time: e.start_time,
            duration_minutes: e.duration_minutes,
        }
    }).collect())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<VantageConfig, String> {
    let cfg = state.config.lock().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn save_config(config: VantageConfig, state: State<'_, AppState>) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    *state.config.lock().await = config;
    Ok(())
}
