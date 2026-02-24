
pub fn agent_from_uuid(uuid: &str) -> String {
    let known = match uuid.to_lowercase().as_str() {
        "e370fa57-4757-3604-3648-499e1f642d3f" => "Gekko",
        "dade69b4-4f5a-8528-247b-219e5a1facd6" => "Fade",
        "5f8d3a7f-467b-97f3-062c-13acf203c006" => "Breach",
        "cc8b64c8-4b25-4ff9-6e7f-37b4da43d235" => "Deadlock",
        "f94c3b30-42be-e959-889c-5aa313dba261" => "Raze",
        "22697a3d-45bf-8dd7-4fec-84a9e28c69d7" => "Chamber",
        "601dbbe7-43ce-be57-2a40-4abd24953621" => "KAY/O",
        "6f2a04ca-43e0-be17-7f36-b3908627744d" => "Skye",
        "117ed9e3-49f3-6512-3ccf-0cada7e3823b" => "Cypher",
        "320b2a48-4d9b-a075-30f1-1f93a9b638fa" => "Sova",
        "1e58de9c-4950-5125-93e9-a0aee9f98746" => "Killjoy",
        "95b78ed7-4637-86d9-7e41-71ba8c293152" => "Harbor",
        "7f94d92c-4234-0a36-9646-3a87eb8b5eef" => "Viper",
        "eb93336a-449b-9c1b-0a54-a891f7921d69" => "Phoenix",
        "41fb69c1-4189-7b37-f117-bcaf1e96f1bf" => "Astra",
        "9c0a79b2-4170-be4c-9e1a-496a65f3f069" => "Sage",
        "a3bfb853-43b2-7238-a4f1-ad90e9e46bcc" => "Reyna",
        "bb2a4828-46eb-8cd1-e765-15848195d751" => "Neon",
        "0e38b510-41a8-5780-7e1f-7e815bf6d1f6" => "Yoru",
        "8e253930-4c05-31dd-1b6c-968525494517" => "Omen",
        "1dbf2edd-4729-0984-3115-daa5eed44993" => "Clove",
        "efba5359-4016-a1e5-7626-b1ae7098ed95" => "Iso",
        "add6443a-41bd-e414-f6ad-e58d267f4e95" => "Jett",
        "f0767a91-4cda-4ff5-b5c0-e57a145b01af" => "Tejo",
        "569fdd95-4d10-43ab-ca70-79becc718b46" => "Sage",
        "707eab51-4836-f488-046a-cda6bf494859" => "Viper",
        "5295c116-4749-0516-4b89-e0a4de94d37a" => "Yoru",
        _ => "",
    };
    if known.is_empty() && !uuid.is_empty() {
        // Unknown UUID — print short form so we can add it to the map
        eprintln!("  [?] Unknown agent UUID: {}", uuid);
    }
    known.to_string()
}

#[derive(Debug, Clone)]
pub struct SmurfFlags {
    pub score: u8,
    #[allow(dead_code)] pub low_level: bool,
    #[allow(dead_code)] pub high_hs: bool,
    #[allow(dead_code)] pub high_wr: bool,
    #[allow(dead_code)] pub rank_spike: bool,
}

impl SmurfFlags {
    pub fn analyze(account_level: u32, rank: u32, hs: f32, wins: u32, total: u32) -> Self {
        let mut score = 0u8;
        let wr = if total > 0 { wins as f32 / total as f32 * 100.0 } else { 0.0 };

        let low_level  = account_level > 0 && account_level < 50;
        let high_hs    = hs >= 30.0;
        let high_wr    = wr >= 65.0 && total >= 5;
        let rank_spike = rank <= 5 && (high_hs || high_wr);

        if low_level  { score += 30; }
        if high_hs    { score += 25; }
        if high_wr    { score += 25; }
        if rank_spike { score += 20; }

        Self { score: score.min(100), low_level, high_hs, high_wr, rank_spike }
    }

    pub fn label(&self) -> Option<&'static str> {
        match self.score {
            75..=100 => Some("⚠ SMURF"),
            50..=74  => Some("? Sus"),
            _        => None,
        }
    }
}
