use colored::*;
use crate::analysis::SmurfFlags;

pub fn rank_from_tier(tier: u32) -> &'static str {
    match tier {
        0  => "Unranked",
        3  => "Iron 1",      4  => "Iron 2",      5  => "Iron 3",
        6  => "Bronze 1",    7  => "Bronze 2",     8  => "Bronze 3",
        9  => "Silver 1",    10 => "Silver 2",     11 => "Silver 3",
        12 => "Gold 1",      13 => "Gold 2",       14 => "Gold 3",
        15 => "Plat 1",      16 => "Plat 2",       17 => "Plat 3",
        18 => "Diamond 1",   19 => "Diamond 2",    20 => "Diamond 3",
        21 => "Ascendant 1", 22 => "Ascendant 2",  23 => "Ascendant 3",
        24 => "Immortal 1",  25 => "Immortal 2",   26 => "Immortal 3",
        27 => "Radiant",
        _  => "Unknown",
    }
}

pub fn colorize_rank(rank: &str) -> ColoredString {
    if rank.contains("Iron")       { return rank.truecolor(110, 105, 105); }
    if rank.contains("Bronze")     { return rank.truecolor(150, 100, 60); }
    if rank.contains("Silver")     { return rank.truecolor(180, 180, 190); }
    if rank.contains("Gold")       { return rank.truecolor(220, 180, 50); }
    if rank.contains("Plat")       { return rank.truecolor(50, 200, 180); }
    if rank.contains("Diamond")    { return rank.truecolor(100, 150, 255); }
    if rank.contains("Ascendant")  { return rank.truecolor(50, 220, 100); }
    if rank.contains("Immortal")   { return rank.truecolor(200, 50, 80); }
    if rank.contains("Radiant")    { return rank.truecolor(255, 220, 80).bold(); }
    rank.white()
}

pub fn wr_color(wins: u32, total: u32) -> ColoredString {
    if total == 0 { return "N/A WR".dimmed(); }
    let pct = wins as f32 / total as f32 * 100.0;
    let s = format!("{:.0}% WR ({}/{})", pct, wins, total);
    if pct >= 55.0      { s.bright_green() }
    else if pct >= 45.0 { s.white() }
    else                { s.truecolor(200, 80, 80) }
}

pub fn avg_rank_label(tiers: &[u32]) -> String {
    let ranked: Vec<u32> = tiers.iter().copied().filter(|&t| t > 0).collect();
    if ranked.is_empty() { return "Avg: N/A".to_string(); }
    let avg = ranked.iter().sum::<u32>() / ranked.len() as u32;
    format!("Avg: {}", rank_from_tier(avg))
}

pub fn print_header() {
    println!();
    println!("{}", "╔══════════════════════════════════════╗".bright_red());
    println!("{}", "║           VANTAGE  v0.1.0            ║".bright_red());
    println!("{}", "╚══════════════════════════════════════╝".bright_red());
    println!();
}

pub fn print_separator() {
    println!("{}", "──────────────────────────────────────────────────────────────────".dimmed());
}

// PlayerRow: (name, tag, agent, tier, wins, total, hs, level, peak_rank, peak_act, is_self)
pub fn print_team(
    label: &str,
    players: &[(String, String, String, u32, u32, u32, f32, u32, u32, String, bool)],
    avg: &str,
) {
    println!("\n  {} {}  {}", "▶".bright_red(), label.bold(), avg.dimmed());
    print_separator();

    for (name, tag, agent, tier, wins, total, hs, level, peak_rank, peak_act, is_self) in players {
        let rank     = rank_from_tier(*tier);
        let rank_col = colorize_rank(rank);
        let wr_col   = wr_color(*wins, *total);

        let hs_str: ColoredString = if *hs > 0.0 {
            let s = format!("{:.0}% HS", hs);
            if *hs >= 25.0      { s.bright_green() }
            else if *hs >= 15.0 { s.white() }
            else                { s.truecolor(200, 80, 80) }
        } else {
            "N/A HS".dimmed()
        };

        // Peak rank + act
        let peak_str: ColoredString = if *peak_rank > 0 {
            let pr = rank_from_tier(*peak_rank);
            let act = if peak_act.is_empty() {
                String::new()
            } else {
                format!(" ({})", peak_act)
            };
            let s = format!("↑{}{}", pr, act);
            colorize_rank(&s)
        } else {
            "".dimmed()
        };

        // Smurf detection
        let smurf = SmurfFlags::analyze(*level, *tier, *hs, *wins, *total);
        let smurf_str: ColoredString = match smurf.label() {
            Some("⚠ SMURF") => "⚠ SMURF".bright_red().bold(),
            Some("? Sus")    => "? Sus".yellow(),
            _                => "".white(),
        };

        let agent_str: ColoredString = if agent.is_empty() {
            "          ".dimmed()
        } else {
            format!("[{:<10}]", agent).truecolor(180, 180, 180)
        };

        // Anonymous = jméno není dostupné, zobraz agenta jako jméno
        let (display_name, display_tag): (ColoredString, ColoredString) = if name.is_empty() {
            if *is_self {
                (format!("★ ???").bright_yellow().bold(), "".dimmed())
            } else if !agent.is_empty() {
                (format!("~ {}", agent).truecolor(140, 140, 140), "hidden".dimmed())
            } else {
                ("~ Anonymous".truecolor(100, 100, 100), "".dimmed())
            }
        } else if *is_self {
            (format!("★ {}", name).bright_yellow().bold(), tag.white())
        } else {
            (name.white().into(), tag.dimmed())
        };

        let level_str: ColoredString = if *level > 0 {
            format!("Lv{:<4}", level).dimmed()
        } else {
            "      ".dimmed()
        };

        println!(
            "  {:<24} {:<10} {} {:<16} {:<18} {:<8} {} {:<24} {}",
            display_name, display_tag,
            agent_str, rank_col, wr_col, hs_str,
            level_str, peak_str, smurf_str,
        );
    }
}

pub fn print_post_game(summary: &crate::models::MatchSummary, my_puuid: &str) {
    // Header box
    let result_str = if summary.won {
        "WIN".bright_green().bold()
    } else {
        "LOSS".bright_red().bold()
    };
    let score_str = format!("{}-{}", summary.rounds_won, summary.rounds_lost);

    let mode_display = match summary.mode.as_str() {
        "competitive" => "Competitive",
        "unrated"     => "Unrated",
        "swiftplay"   => "Swiftplay",
        "spikerush"   => "Spike Rush",
        "deathmatch"  => "Deathmatch",
        other         => other,
    };

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_red());
    println!("{}  POST-GAME  │  {}  │  {}  │  {} {}  {}",
        "║".bright_red(),
        summary.map.bold(),
        mode_display.dimmed(),
        score_str.bold(),
        result_str,
        "║".bright_red(),
    );
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_red());
    println!();

    // Table header
    println!("  {:<20} {:<12} {:<10} {:<6} {:<6} {:<6} {:<4}",
        "PLAYER".bold(), "AGENT".bold(), "K/D/A".bold(),
        "ACS".bold(), "HS%".bold(), "DMG".bold(), "FB".bold());
    print_separator();

    // Split into my team and enemy team
    let my_team = summary.all_players.iter()
        .find(|p| p.puuid == my_puuid)
        .map(|p| p.team.clone())
        .unwrap_or_else(|| "Blue".to_string());

    let mut allies: Vec<_> = summary.all_players.iter().filter(|p| p.team == my_team).collect();
    let mut enemies: Vec<_> = summary.all_players.iter().filter(|p| p.team != my_team).collect();

    // Sort by ACS (descending)
    allies.sort_by(|a, b| b.score.cmp(&a.score));
    enemies.sort_by(|a, b| b.score.cmp(&a.score));

    // Print allies
    for p in &allies {
        print_player_row(p, p.puuid == my_puuid);
    }
    print_separator();

    // Print enemies
    for p in &enemies {
        print_player_row(p, false);
    }

    println!();
}

fn print_player_row(p: &crate::models::PlayerMatchStats, is_self: bool) {
    let name_display = if is_self {
        format!("★ {}", p.name).bright_yellow().bold()
    } else {
        p.name.white().into()
    };

    let kda = format!("{}/{}/{}", p.kills, p.deaths, p.assists);
    let acs = format!("{}", p.score);

    let hs_str = format!("{:.0}%", p.hs_percent);
    let hs_col: ColoredString = if p.hs_percent >= 25.0 {
        hs_str.bright_green()
    } else if p.hs_percent >= 15.0 {
        hs_str.white()
    } else {
        hs_str.truecolor(200, 80, 80)
    };

    let dmg = format!("{}", p.damage_dealt);
    let fb = format!("{}", p.first_bloods);

    let agent_col = if p.agent.is_empty() {
        "???".dimmed()
    } else {
        p.agent.truecolor(180, 180, 180).into()
    };

    println!("  {:<20} {:<12} {:<10} {:<6} {:<6} {:<6} {:<4}",
        name_display, agent_col, kda, acs, hs_col, dmg, fb);
}
