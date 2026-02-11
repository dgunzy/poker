import { useState } from "react";
import { theme, RANK_DISPLAY } from "../theme";
import { CardDisplay } from "./CardDisplay";

export interface CardSelection {
  rank: string;
  suit: string;
  index: number;
}

interface CardPickerProps {
  selected: CardSelection[];
  onChange: (cards: CardSelection[]) => void;
  maxCards: number;
  excludedCards: CardSelection[];
}

const RANKS = ["two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "jack", "queen", "king", "ace"];
const SUITS = ["clubs", "diamonds", "hearts", "spades"];

function isExcluded(card: { rank: string; suit: string }, excluded: CardSelection[]) {
  return excluded.some((e) => e.rank === card.rank && e.suit === card.suit);
}

function isSelected(card: { rank: string; suit: string }, selected: CardSelection[]) {
  return selected.some((s) => s.rank === card.rank && s.suit === card.suit);
}

function rankLabel(rank: string): string {
  return RANK_DISPLAY[rank] ?? rank[0]?.toUpperCase() ?? "?";
}

export function CardPicker({ selected, onChange, maxCards, excludedCards }: CardPickerProps) {
  const [showPicker, setShowPicker] = useState(false);

  const toggleCard = (rank: string, suit: string) => {
    const card = { rank, suit };
    if (isSelected(card, selected)) {
      onChange(selected.filter((s) => !(s.rank === rank && s.suit === suit)));
    } else if (selected.length < maxCards && !isExcluded(card, excludedCards)) {
      onChange([...selected, { rank, suit, index: selected.length }]);
    }
  };

  return (
    <div>
      <div style={styles.selectedRow}>
        {selected.map((c, i) => (
          <div
            key={`${c.rank}-${c.suit}-${i}`}
            onClick={() => onChange(selected.filter((_, j) => j !== i))}
            style={styles.selectableCard}
            title={`${rankLabel(c.rank)} of ${c.suit}`}
          >
            <CardDisplay card={c} size="medium" />
          </div>
        ))}
        {selected.length < maxCards && (
          <button
            onClick={() => setShowPicker(!showPicker)}
            style={{
              ...styles.addButton,
              borderColor: showPicker ? theme.primary : theme.border,
            }}
          >
            +
          </button>
        )}
      </div>
      {showPicker && (
        <div style={styles.grid}>
          {RANKS.flatMap((rank) =>
            SUITS.map((suit) => {
              const sel = isSelected({ rank, suit }, selected);
              const excl = isExcluded({ rank, suit }, excludedCards);
              const cannotAdd = excl || (!sel && selected.length >= maxCards);
              const disabled = !sel && cannotAdd;
              return (
                <button
                  key={`${rank}-${suit}`}
                  onClick={() => toggleCard(rank, suit)}
                  disabled={disabled}
                  style={{
                    ...styles.gridCard,
                    opacity: disabled ? 0.5 : 1,
                    cursor: disabled ? "not-allowed" : "pointer",
                  }}
                  title={disabled ? (excl ? "Card in use" : "Max cards reached") : `${rankLabel(rank)} ${suit}`}
                >
                  <CardDisplay card={{ rank, suit }} size="small" />
                </button>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  selectedRow: {
    display: "flex",
    gap: 6,
    flexWrap: "wrap",
    marginBottom: 8,
    alignItems: "center",
  },
  selectableCard: {
    cursor: "pointer",
    boxShadow: "0 1px 3px rgba(0,0,0,0.3)",
    borderRadius: 6,
    overflow: "hidden",
  },
  addButton: {
    width: 48,
    height: 64,
    border: `2px dashed ${theme.border}`,
    borderRadius: 6,
    background: "transparent",
    color: theme.textMuted,
    cursor: "pointer",
    fontSize: 24,
  },
  grid: {
    display: "flex",
    flexWrap: "wrap",
    gap: 6,
    marginTop: 8,
  },
  gridCard: {
    padding: 0,
    border: `1px solid ${theme.border}`,
    borderRadius: 6,
    overflow: "hidden",
    background: "transparent",
  },
};
