/** Theme tokens from AGENTS.md config/ui.toml */

export const theme = {
  suits: {
    clubs: "#2d5a27",
    diamonds: "#1e3a8a",
    hearts: "#991b1b",
    spades: "#374151",
  },
  feedback: {
    correct: "#22c55e",
    close: "#eab308",
    incorrect: "#ef4444",
  },
  card: {
    width: 60,
    height: 84,
    fontSize: 18,
  },
  background: "#0f172a",
  surface: "#1e293b",
  surfaceHover: "#334155",
  text: "#f1f5f9",
  textMuted: "#94a3b8",
  border: "#334155",
  primary: "#3b82f6",
  feedbackCorrect: "#22c55e",
  feedbackClose: "#eab308",
  feedbackIncorrect: "#ef4444",
  fontFamily: "ui-sans-serif, system-ui, -apple-system, sans-serif",
  sidebarWidth: 200,
  headerHeight: 48,
};

export const RANK_DISPLAY: Record<string, string> = {
  two: "2",
  three: "3",
  four: "4",
  five: "5",
  six: "6",
  seven: "7",
  eight: "8",
  nine: "9",
  ten: "T",
  jack: "J",
  queen: "Q",
  king: "K",
  ace: "A",
};
