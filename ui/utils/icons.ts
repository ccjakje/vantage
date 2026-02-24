// Mapování jméno agenta → ikona
export function agentIcon(name: string): string {
  if (!name) return "";
  const overrides: Record<string, string> = {
    "KAY/O": "KAYO",
    "KAYO":  "KAYO",
    "Veto":  "Veto",
  };
  const normalized = overrides[name] ?? name;
  return `/agents/${normalized}_icon.webp`;
}

// Mapování tier číslo → rank ikona
export function rankIcon(tier: number): string {
  if (tier === 0) return "";
  const name = rankIconName(tier);
  return name ? `/ranks/${name}.png` : "";
}

function rankIconName(tier: number): string {
  const map: Record<number, string> = {
    3:  "Iron_1_Rank",      4:  "Iron_2_Rank",      5:  "Iron_3_Rank",
    6:  "Bronze_1_Rank",    7:  "Bronze_2_Rank",     8:  "Bronze_3_Rank",
    9:  "Silver_1_Rank",    10: "Silver_2_Rank",     11: "Silver_3_Rank",
    12: "Gold_1_Rank",      13: "Gold_2_Rank",       14: "Gold_3_Rank",
    15: "Platinum_1_Rank",  16: "Platinum_2_Rank",   17: "Platinum_3_Rank",
    18: "Diamond_1_Rank",   19: "Diamond_2_Rank",    20: "Diamond_3_Rank",
    21: "Ascendant_1_Rank", 22: "Ascendant_2_Rank",  23: "Ascendant_3_Rank",
    24: "Immortal_1_Rank",  25: "Immortal_2_Rank",   26: "Immortal_3_Rank",
    27: "Radiant_Rank",
  };
  return map[tier] ?? "";
}

// Rank label z tier čísla
export function rankLabel(tier: number): string {
  const map: Record<number, string> = {
    0:  "Unranked",
    3:  "Iron 1",      4:  "Iron 2",      5:  "Iron 3",
    6:  "Bronze 1",    7:  "Bronze 2",    8:  "Bronze 3",
    9:  "Silver 1",    10: "Silver 2",    11: "Silver 3",
    12: "Gold 1",      13: "Gold 2",      14: "Gold 3",
    15: "Plat 1",      16: "Plat 2",      17: "Plat 3",
    18: "Diamond 1",   19: "Diamond 2",   20: "Diamond 3",
    21: "Ascendant 1", 22: "Ascendant 2", 23: "Ascendant 3",
    24: "Immortal 1",  25: "Immortal 2",  26: "Immortal 3",
    27: "Radiant",
  };
  return map[tier] ?? "Unknown";
}

// Rank barva
export function rankColor(tier: number): string {
  if (tier <= 0)  return "#888888";
  if (tier <= 5)  return "#6e6a6a";
  if (tier <= 8)  return "#a0633c";
  if (tier <= 11) return "#b4b4c0";
  if (tier <= 14) return "#ddb432";
  if (tier <= 17) return "#32c8b4";
  if (tier <= 20) return "#6496ff";
  if (tier <= 23) return "#32dc64";
  if (tier <= 26) return "#c83250";
  return "#ffe050";
}

// Agent class
export type AgentClass = "Duelist" | "Initiator" | "Controller" | "Sentinel" | "";

export function agentClass(name: string): AgentClass {
  const duelists    = ["Jett", "Reyna", "Raze", "Neon", "Yoru", "Phoenix", "Iso", "Waylay"];
  const initiators  = ["Sova", "Breach", "Skye", "KAY/O", "Fade", "Gekko"];
  const controllers = ["Brimstone", "Omen", "Viper", "Astra", "Harbor", "Clove"];
  const sentinels   = ["Sage", "Cypher", "Killjoy", "Chamber", "Deadlock", "Vyse", "Tejo", "Veto"];
  if (duelists.includes(name))    return "Duelist";
  if (initiators.includes(name))  return "Initiator";
  if (controllers.includes(name)) return "Controller";
  if (sentinels.includes(name))   return "Sentinel";
  return "";
}

export function classIcon(cls: string): string {
  if (!cls) return "";
  return `/class_symbols/${cls}ClassSymbol.webp`;
}

// Map codename → display name
export function mapName(raw: string): string {
  const names: Record<string, string> = {
    Infinity:    "Abyss",
    Ascent:      "Ascent",
    Duality:     "Bind",
    Foxtrot:     "Breeze",
    Canyon:      "Fracture",
    Triad:       "Haven",
    Port:        "Icebox",
    Jam:         "Lotus",
    Pitt:        "Pearl",
    Bonsai:      "Split",
    Juliett:     "Sunset",
    HURM_Alley:  "District",
    HURM_Helix:  "Drift",
    HURM_Bowl:   "Kasbah",
    HURM_Yard:   "Piazza",
    Poveglia:    "The Range",
    Skirmish_A:  "Skirmish A",
    Skirmish_C:  "Skirmish C",
  };
  return names[raw] ?? raw;
}
