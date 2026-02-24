import { agentIcon, rankIcon, rankLabel, rankColor, classIcon, agentClass } from "../utils/icons";

// Agent ikona s fallback na initials
export function AgentIcon({ name, size = 28 }: { name: string; size?: number }) {
  const src = agentIcon(name);
  if (!src) return (
    <div
      className="rounded flex items-center justify-center bg-[var(--bg-card-hover)] text-[var(--text-muted)] font-bold shrink-0"
      style={{ width: size, height: size, fontSize: size * 0.4 }}
    >
      ?
    </div>
  );
  return (
    <img
      src={src}
      alt={name}
      width={size}
      height={size}
      className="rounded object-cover shrink-0"
      onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
    />
  );
}

// Rank ikona s label pod ní (volitelný)
export function RankIcon({
  tier,
  size = 28,
  showLabel = false,
}: {
  tier: number;
  size?: number;
  showLabel?: boolean;
}) {
  const src = rankIcon(tier);
  const label = rankLabel(tier);
  const color = rankColor(tier);

  if (!src) return (
    <span className="text-xs" style={{ color }}>
      {label}
    </span>
  );

  return (
    <div className="flex flex-col items-center gap-0.5">
      <img
        src={src}
        alt={label}
        width={size}
        height={size}
        className="object-contain shrink-0"
        onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
      />
      {showLabel && (
        <span className="text-xs font-medium leading-none" style={{ color }}>
          {label}
        </span>
      )}
    </div>
  );
}

// Inline rank — ikona + text vedle sebe
export function RankInline({ tier, size = 20 }: { tier: number; size?: number }) {
  const src = rankIcon(tier);
  const label = rankLabel(tier);
  const color = rankColor(tier);

  return (
    <div className="flex items-center gap-1.5">
      {src && (
        <img
          src={src}
          alt={label}
          width={size}
          height={size}
          className="object-contain shrink-0"
          onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
        />
      )}
      <span className="text-sm font-medium leading-none" style={{ color }}>
        {label}
      </span>
    </div>
  );
}

// Class symbol badge
export function ClassBadge({ agent, size = 16 }: { agent: string; size?: number }) {
  const cls = agentClass(agent);
  const src = classIcon(cls);
  if (!src) return null;
  return (
    <img
      src={src}
      alt={cls}
      title={cls}
      width={size}
      height={size}
      className="object-contain opacity-60 shrink-0"
    />
  );
}
