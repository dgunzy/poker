import { useState, useEffect } from "react";
import { getGames } from "../api";
import { theme } from "../theme";

export interface Game {
  id: string;
  name: string;
  hand_size: number;
  deck_size: number;
  description: string;
}

interface GameSelectorProps {
  value: string;
  onChange: (id: string) => void;
}

export function GameSelector({ value, onChange }: GameSelectorProps) {
  const [games, setGames] = useState<Game[]>([]);

  useEffect(() => {
    getGames().then(setGames).catch(() => {});
  }, []);

  if (games.length === 0) return null;
  const currentValue = value || games[0]?.id;
  if (games.length === 1) {
    return (
      <div style={styles.single}>
        <span style={styles.label}>{games[0].name}</span>
      </div>
    );
  }

  return (
    <select
      value={currentValue}
      onChange={(e) => onChange(e.target.value)}
      style={styles.select}
    >
      {games.map((g) => (
        <option key={g.id} value={g.id}>
          {g.name}
        </option>
      ))}
    </select>
  );
}

const styles: Record<string, React.CSSProperties> = {
  single: {
    fontSize: "0.9rem",
    color: theme.textMuted,
  },
  label: {
    fontWeight: 500,
  },
  select: {
    padding: "0.35rem 0.5rem",
    borderRadius: 4,
    border: `1px solid ${theme.border}`,
    background: theme.surface,
    color: theme.text,
    fontSize: "0.9rem",
    width: "100%",
    maxWidth: "100%",
    minWidth: 0,
    boxSizing: "border-box",
  },
};
