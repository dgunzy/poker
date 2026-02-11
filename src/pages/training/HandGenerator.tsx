import { useState, useEffect } from "react";
import { runSimulation as runSimulationApi, getPreferences, setPreferences } from "../../api";
import { useGame } from "../../context/GameContext";
import { CardPicker } from "../../components/CardPicker";
import { CardDisplay } from "../../components/CardDisplay";
import { theme } from "../../theme";

export interface CardSelection {
  rank: string;
  suit: string;
  index: number;
}

export function HandGenerator() {
  const { gameId } = useGame();
  const heldMaxCards = gameId === "holdem" ? 2 : gameId === "omaha" || gameId === "omaha8" ? 4 : 5;
  const [heldCards, setHeldCards] = useState<CardSelection[]>([]);
  const [deadCards, setDeadCards] = useState<CardSelection[]>([]);
  const [drawCount, setDrawCount] = useState(100);

  useEffect(() => {
    getPreferences()
      .then((p) => {
        if (p.training_draw_count >= 1) setDrawCount(p.training_draw_count);
      })
      .catch(() => {});
  }, []);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentHand, setCurrentHand] = useState<CardSelection[] | null>(null);
  const [percentile, setPercentile] = useState<number | null>(null);

  const generateHand = async () => {
    setLoading(true);
    setError(null);
    setCurrentHand(null);
    setPercentile(null);
    try {
      getPreferences().then((p) => setPreferences({ ...p, training_draw_count: drawCount })).catch(() => {});
      const input = {
        held_cards: heldCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        dead_cards: deadCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        draw_count: Math.max(drawCount, 1),
        game_id: gameId,
      };
      const res = await runSimulationApi(input);
      if (res.results.length === 0) {
        setError("No results from simulation");
        return;
      }
      const idx = Math.floor(Math.random() * res.results.length);
      const r = res.results[idx];
      setCurrentHand(r.hand.map((c, i) => ({ ...c, index: i })));
      setPercentile(r.percentile);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const isDrawGame = gameId === "deuce_to_seven" || gameId === "ace_to_five";
  const heldLabel = isDrawGame ? "Your draw (optional)" : "Hole cards";

  return (
    <div style={styles.container}>
      <section style={styles.section}>
        <h2 style={styles.sectionTitle}>Setup</h2>
        <p style={styles.hint}>
          {isDrawGame
            ? "Set your draw and dead cards for context, or leave empty for purely random 5-card hands. Great for flash cards."
            : "Set 0–2 hole cards (Hold'em) or 0–4 (Omaha). We pad with random cards and simulate boards. Dead cards excluded."}
        </p>
        <div style={styles.setupRow}>
          <div>
            <h3 style={styles.subTitle}>{heldLabel}</h3>
            <CardPicker selected={heldCards} onChange={setHeldCards} maxCards={heldMaxCards} excludedCards={deadCards} />
          </div>
          <div>
            <h3 style={styles.subTitle}>Dead cards</h3>
            <CardPicker selected={deadCards} onChange={setDeadCards} maxCards={20} excludedCards={heldCards} />
          </div>
        </div>
        <label style={styles.label}>
          Pool size:
          <input
            type="number"
            value={drawCount}
            onChange={(e) => setDrawCount(Math.max(1, Math.min(10000, parseInt(e.target.value) || 100)))}
            min={1}
            max={10000}
            style={styles.input}
          />
        </label>
        <button
          onClick={generateHand}
          disabled={loading}
          style={{ ...styles.button, opacity: loading ? 0.6 : 1 }}
        >
          {loading ? "Generating…" : "Generate hand"}
        </button>
      </section>

      {error && <div style={styles.error}>{error}</div>}

      {currentHand && (
        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Hand</h2>
          <div style={styles.handRow}>
            {currentHand.map((c, i) => (
              <CardDisplay key={i} card={c} size="normal" />
            ))}
          </div>
          {percentile !== null && (
            <p style={styles.percentile}>Percentile: {percentile.toFixed(1)}%</p>
          )}
          <button onClick={generateHand} style={styles.nextButton}>
            Next hand
          </button>
        </section>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {},
  section: { marginBottom: "2rem" },
  sectionTitle: { margin: "0 0 0.5rem", fontSize: "1rem", fontWeight: 600 },
  subTitle: { margin: "0 0 0.5rem", fontSize: "0.9rem", fontWeight: 500 },
  hint: { margin: "0 0 0.75rem", fontSize: "0.8rem", color: theme.textMuted },
  setupRow: { display: "flex", gap: "2rem", flexWrap: "wrap", marginBottom: "1rem" },
  label: { display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.75rem" },
  input: {
    padding: "0.35rem 0.5rem",
    borderRadius: 4,
    border: `1px solid ${theme.border}`,
    background: theme.surface,
    color: theme.text,
  },
  button: {
    padding: "0.5rem 1rem",
    borderRadius: 6,
    border: "none",
    background: theme.primary,
    color: "white",
    fontWeight: 600,
    cursor: "pointer",
  },
  error: {
    padding: "0.75rem",
    background: theme.feedbackIncorrect,
    color: "white",
    borderRadius: 6,
    marginBottom: "1rem",
  },
  handRow: { display: "flex", gap: 8, marginBottom: "1rem", flexWrap: "wrap" },
  percentile: { fontSize: "0.9rem", color: theme.textMuted, marginBottom: "1rem" },
  nextButton: {
    padding: "0.5rem 1rem",
    borderRadius: 6,
    border: `1px solid ${theme.border}`,
    background: theme.surface,
    color: theme.text,
    cursor: "pointer",
  },
};
