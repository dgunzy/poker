import { theme, RANK_DISPLAY } from "../theme";

export interface CardDisplayProps {
  card: { rank: string; suit: string };
  /** xs: results table; small: picker grid; medium: selected cards; normal: full card */
  size?: "xs" | "small" | "medium" | "normal";
}

const SUIT_SYMBOLS: Record<string, string> = {
  clubs: "♣",
  diamonds: "♦",
  hearts: "♥",
  spades: "♠",
};

const SIZE_DIMS: Record<string, { w: number; h: number; fs: number; suitFs: number }> = {
  xs: { w: 36, h: 50, fs: 12, suitFs: 8 },
  small: { w: 44, h: 58, fs: 16, suitFs: 10 },
  medium: { w: 48, h: 64, fs: 18, suitFs: 11 },
  normal: { w: theme.card.width, h: theme.card.height, fs: theme.card.fontSize, suitFs: 12 },
};

export function CardDisplay({ card, size = "normal" }: CardDisplayProps) {
  const suitColor = theme.suits[card.suit as keyof typeof theme.suits] ?? theme.surface;
  const rankDisplay = RANK_DISPLAY[card.rank] ?? card.rank;
  const suitSymbol = SUIT_SYMBOLS[card.suit] ?? "";
  const dims = SIZE_DIMS[size] ?? SIZE_DIMS.normal;

  return (
    <div
      style={{
        width: dims.w,
        height: dims.h,
        background: suitColor,
        borderRadius: 6,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: dims.fs,
        fontWeight: 700,
        color: "white",
        position: "relative",
      }}
    >
      <span
        style={{
          position: "absolute",
          top: 2,
          left: 3,
          fontSize: dims.suitFs,
          opacity: 0.95,
          lineHeight: 1,
        }}
      >
        {suitSymbol}
      </span>
      <span
        style={{
          position: "absolute",
          bottom: 2,
          right: 3,
          fontSize: dims.suitFs,
          opacity: 0.95,
          lineHeight: 1,
        }}
      >
        {suitSymbol}
      </span>
      {rankDisplay}
    </div>
  );
}
