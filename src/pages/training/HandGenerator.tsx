import { useState, useEffect } from "react";
import {
  getPreferences,
  setPreferences,
  getGameTrainingInfo,
  generateTrainingScenario,
  type GameTrainingInfo,
  type TrainingScenario,
} from "../../api";
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
  const [trainingInfo, setTrainingInfo] = useState<GameTrainingInfo | null>(null);
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

  useEffect(() => {
    getGameTrainingInfo(gameId).then(setTrainingInfo).catch(() => {});
    setHeldCards([]);
    setDeadCards([]);
    setScenario(null);
  }, [gameId]);

  const heldMaxCards = trainingInfo?.max_held ?? 5;

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [scenario, setScenario] = useState<TrainingScenario | null>(null);

  const generateHand = async () => {
    setLoading(true);
    setError(null);
    setScenario(null);
    try {
      getPreferences().then((p) => setPreferences({ ...p, training_draw_count: drawCount })).catch(() => {});
      const result = await generateTrainingScenario({
        held_cards: heldCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        dead_cards: deadCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        sim_count: Math.max(drawCount, 1),
        game_id: gameId,
      });
      setScenario(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const renderHandDisplay = () => {
    if (!scenario) return null;

    if (scenario.type === "draw") {
      return (
        <div>
          <div style={styles.handRow}>
            {scenario.hand.map((c, i) => (
              <CardDisplay key={i} card={c} size="normal" />
            ))}
          </div>
          <p style={styles.percentile}>Percentile: {scenario.percentile.toFixed(1)}%</p>
        </div>
      );
    }

    const holeCards = scenario.hole_cards;
    const board = scenario.board;
    const highPercentile = scenario.type === "board" ? scenario.percentile : scenario.high_percentile;

    return (
      <div>
        <h3 style={styles.subTitle}>Hole cards</h3>
        <div style={styles.handRow}>
          {holeCards.map((c, i) => (
            <CardDisplay key={`hole-${i}`} card={c} size="normal" />
          ))}
        </div>
        <h3 style={{ ...styles.subTitle, marginTop: "0.75rem" }}>Board</h3>
        <div style={styles.handRow}>
          {board.map((c, i) => (
            <CardDisplay key={`board-${i}`} card={c} size="normal" />
          ))}
        </div>
        <p style={styles.percentile}>
          {scenario.type === "split_pot" ? "High percentile" : "Percentile"}: {highPercentile.toFixed(1)}%
        </p>
        {scenario.type === "split_pot" && (
          scenario.low_qualifies && scenario.low_percentile !== null ? (
            <p style={styles.percentile}>Low percentile: {scenario.low_percentile.toFixed(1)}%</p>
          ) : (
            <div style={styles.noLowBadge}>No qualifying low</div>
          )
        )}
      </div>
    );
  };

  return (
    <div style={styles.container}>
      <section style={styles.section}>
        <h2 style={styles.sectionTitle}>Setup</h2>
        <p style={styles.hint}>{trainingInfo?.held_hint ?? ""}</p>
        <div style={styles.setupRow}>
          <div>
            <h3 style={styles.subTitle}>{trainingInfo?.held_label ?? "Cards"}</h3>
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
          {loading ? "Generating\u2026" : "Generate hand"}
        </button>
      </section>

      {error && <div style={styles.error}>{error}</div>}

      {scenario && (
        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Hand</h2>
          {renderHandDisplay()}
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
  percentile: { fontSize: "0.9rem", color: theme.textMuted, marginBottom: "0.5rem" },
  noLowBadge: {
    display: "inline-block",
    padding: "0.3rem 0.6rem",
    borderRadius: 4,
    background: theme.surface,
    border: `1px solid ${theme.border}`,
    color: theme.textMuted,
    fontSize: "0.8rem",
    marginBottom: "0.5rem",
  },
  nextButton: {
    padding: "0.5rem 1rem",
    borderRadius: 6,
    border: `1px solid ${theme.border}`,
    background: theme.surface,
    color: theme.text,
    cursor: "pointer",
  },
};
