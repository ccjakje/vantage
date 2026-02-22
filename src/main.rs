mod lockfile;
mod models;
mod local_api;
mod display;
mod analysis;

use anyhow::Result;
use colored::*;
use std::time::Duration;

use lockfile::read_lockfile;
use local_api::LocalClient;
use display::{print_header, print_separator, print_team, avg_rank_label};
use analysis::SmurfFlags;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\n  {} {}\n", "✗".red(), e.to_string().red());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    print_header();

    let watch_mode = std::env::args().any(|a| a == "--watch" || a == "-w");

    println!("  {} Looking for Valorant...", "→".dimmed());
    let lockfile = read_lockfile()?;
    println!("  {} Valorant found (port: {})", "✓".green(), lockfile.port);

    let client = LocalClient::new(lockfile).await?;
    println!("  {} Logged in as: {}", "✓".green(), client.puuid.dimmed());

    print_separator();

    if watch_mode {
        println!("\n  {} Watch mode — waiting for a match...", "◉".bright_red());
        println!("  {}\n", "Press Ctrl+C to stop".dimmed());
        watch_loop(&client).await?;
    } else {
        run_once(&client).await?;
    }

    Ok(())
}

async fn watch_loop(client: &LocalClient) -> Result<()> {
    let mut last_phase = String::new();

    loop {
        let phase = detect_phase(client).await;

        if phase != last_phase {
            match phase.as_str() {
                "pregame" => {
                    println!("\n  {} Agent Select!\n", "◉".bright_yellow());
                    if let Ok(id) = client.get_pregame_match_id().await {
                        if let Err(e) = handle_pregame(client, &id).await {
                            println!("  {} {}", "✗".red(), e);
                        }
                    }
                }
                "coregame" => {
                    println!("\n  {} Match in progress!\n", "◉".bright_green());
                    if let Ok(id) = client.get_coregame_match_id().await {
                        if let Err(e) = handle_coregame(client, &id).await {
                            println!("  {} {}", "✗".red(), e);
                        }
                    }
                }
                "menu" => {
                    if !last_phase.is_empty() {
                        println!("\n  {} Back in menu — waiting...\n", "○".dimmed());
                    }
                }
                _ => {}
            }
            last_phase = phase;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn detect_phase(client: &LocalClient) -> String {
    if client.get_pregame_match_id().await.is_ok() { return "pregame".to_string(); }
    if client.get_coregame_match_id().await.is_ok() { return "coregame".to_string(); }
    "menu".to_string()
}

async fn run_once(client: &LocalClient) -> Result<()> {
    println!("\n  {} Detecting game phase...\n", "→".dimmed());

    if let Ok(id) = client.get_pregame_match_id().await {
        println!("  {} Agent Select detected!", "◉".bright_yellow());
        return handle_pregame(client, &id).await;
    }
    if let Ok(id) = client.get_coregame_match_id().await {
        println!("  {} Match in progress!", "◉".bright_green());
        return handle_coregame(client, &id).await;
    }

    println!("  {} In menu — use --watch to auto-detect\n", "○".dimmed());
    Ok(())
}

// PlayerRow: (name, tag, agent, tier, wins, total, hs, level, peak_rank, peak_act, is_self)
type PlayerRow = (String, String, String, u32, u32, u32, f32, u32, u32, String, bool);

async fn fetch_player_rows(
    client: &LocalClient,
    puuids: &[String],
    agents: Option<&std::collections::HashMap<String, String>>,
) -> Vec<PlayerRow> {
    let names = client.get_player_names(puuids).await.unwrap_or_default();

    let get_name = |puuid: &str| -> (String, String) {
        names.iter()
            .find(|(p, _, _)| p == puuid)
            .map(|(_, n, t)| {
                let name = if n.is_empty() { "Anonymous".to_string() } else { n.clone() };
                let tag  = if t.is_empty() { "????".to_string() } else { t.clone() };
                (name, tag)
            })
            .unwrap_or_else(|| ("Anonymous".to_string(), "????".to_string()))
    };

    let mut rows = Vec::new();
    for puuid in puuids {
        let (rank, wins, total, hs, level, peak_rank, peak_act) = client.get_player_stats(puuid).await
            .unwrap_or((0, 0, 0, 0.0, 0, 0, String::new()));
        let (name, tag) = get_name(puuid);
        let agent = agents
            .and_then(|a| a.get(puuid))
            .cloned()
            .unwrap_or_else(|| "".to_string());
        let is_self = puuid == &client.puuid;
        rows.push((name, tag, agent, rank, wins, total, hs, level, peak_rank, peak_act, is_self));
    }
    rows
}

async fn handle_pregame(client: &LocalClient, match_id: &str) -> Result<()> {
    let match_data = client.get_pregame_match(match_id).await?;

    let mut agents: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let puuids: Vec<String> = match_data.ally_team.players.iter().map(|p| {
        let agent = analysis::agent_from_uuid(&p.character_id).to_string();
        agents.insert(p.subject.clone(), agent);
        p.subject.clone()
    }).collect();

    println!("  {} Fetching stats for {} players...\n", "→".dimmed(), puuids.len());
    let rows = fetch_player_rows(client, &puuids, Some(&agents)).await;
    let tiers: Vec<u32> = rows.iter().map(|r| r.3).collect();
    let avg = avg_rank_label(&tiers);

    print_team("YOUR TEAM (Agent Select)", &rows, &avg);
    println!();
    Ok(())
}

async fn handle_coregame(client: &LocalClient, match_id: &str) -> Result<()> {
    let match_data = client.get_coregame_match(match_id).await?;

    let mut agents: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for p in &match_data.players {
        let agent = analysis::agent_from_uuid(&p.character_id).to_string();
        agents.insert(p.subject.clone(), agent);
    }

    let blue_puuids: Vec<String> = match_data.players.iter()
        .filter(|p| p.team_id == "Blue").map(|p| p.subject.clone()).collect();
    let red_puuids: Vec<String> = match_data.players.iter()
        .filter(|p| p.team_id != "Blue").map(|p| p.subject.clone()).collect();

    println!("  {} Fetching stats for {} players...\n", "→".dimmed(), blue_puuids.len() + red_puuids.len());

    let blue_rows = fetch_player_rows(client, &blue_puuids, Some(&agents)).await;
    let red_rows  = fetch_player_rows(client, &red_puuids,  Some(&agents)).await;

    let blue_avg = avg_rank_label(&blue_rows.iter().map(|r| r.3).collect::<Vec<_>>());
    let red_avg  = avg_rank_label(&red_rows.iter().map(|r| r.3).collect::<Vec<_>>());

    print_team("TEAM BLUE", &blue_rows, &blue_avg);
    print_team("TEAM RED",  &red_rows,  &red_avg);
    println!();
    Ok(())
}
