import { useState, useEffect } from "react";
import { runSimulation as runSimulationApi, getTrainingConfig, getPreferences, setPreferences } from "../../api";
import { useGame } from "../../context/GameContext";
import { CardPicker } from "../../components/CardPicker";
import { CardDisplay } from "../../components/CardDisplay";
import { theme } from "../../theme";

export interface CardSelection {
  rank: string;
  suit: string;
  index: number;
}

type Feedback = "exact" | "close" | "off" | null;

export function PercentileGuessing() {
  const { gameId } = useGame();
  const heldMaxCards = gameId === "holdem" ? 2 : gameId === "omaha" || gameId === "omaha8" ? 4 : 5;
  const [heldCards, setHeldCards] = useState<CardSelection[]>([]);
  const [deadCards, setDeadCards] = useState<CardSelection[]>([]);
  const [drawCount, setDrawCount] = useState(1000);
  const [exactThreshold, setExactThreshold] = useState(4);
  const [closeThreshold, setCloseThreshold] = useState(12);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentHand, setCurrentHand] = useState<CardSelection[] | null>(null);
  const [actualPercentile, setActualPercentile] = useState<number | null>(null);
  const [guess, setGuess] = useState("");
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [attempts, setAttempts] = useState(0);
  const [correct, setCorrect] = useState(0);
  const [streak, setStreak] = useState(0);

  useEffect(() => {
    Promise.all([getTrainingConfig(), getPreferences()]).then(([cfg, prefs]) => {
      setExactThreshold(cfg.exact_threshold);
      setCloseThreshold(cfg.close_threshold);
      setDrawCount(prefs.training_draw_count >= 1 ? prefs.training_draw_count : cfg.default_draw_count);
    }).catch(() => {});
  }, []);

  const getNewHand = async () => {
    setLoading(true);
    setError(null);
    setCurrentHand(null);
    setActualPercentile(null);
    setGuess("");
    setFeedback(null);
    try {
      getPreferences().then((p) => setPreferences({ ...p, training_draw_count: drawCount })).catch(() => {});
      const input = {
        held_cards: heldCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        dead_cards: deadCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        draw_count: drawCount,
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
      setActualPercentile(r.percentile);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const submitGuess = () => {
    const g = parseFloat(guess);
    if (isNaN(g) || actualPercentile === null) return;
    setAttempts((a) => a + 1);
    const diff = Math.abs(g - actualPercentile);
    let fb: Feedback;
    if (diff <= exactThreshold) {
      fb = "exact";
      setCorrect((c) => c + 1);
      setStreak((s) => s + 1);
    } else if (diff <= closeThreshold) {
      fb = "close";
      setStreak(0);
    } else {
      fb = "off";
      setStreak(0);
    }
    setFeedback(fb);
  };

  const accuracy = attempts > 0 ? ((correct / attempts) * 100).toFixed(0) : "—";

  const isDrawGame = gameId === "deuce_to_seven" || gameId === "ace_to_five";
  const heldLabel = isDrawGame ? "Your draw (cards you're keeping)" : "Hole cards";
  const simLabel = isDrawGame ? "Simulations (100 or 1,000 typical)" : "Simulations";
  const heldHint = isDrawGame
    ? "0–5 cards. We simulate random draws to complete 5-card hands. Dead cards excluded."
    : "0–2 (Hold'em) or 0–4 (Omaha). We pad with random cards and simulate boards. Dead cards excluded.";

  return (
    <div style={styles.container}>
      <section style={styles.section}>
        <h2 style={styles.sectionTitle}>Setup</h2>
        <p style={styles.hint}>
          {heldHint}
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
          {simLabel}:
          <input
            type="number"
            value={drawCount}
            onChange={(e) => setDrawCount(Math.max(1, Math.min(10000, parseInt(e.target.value) || 1000)))}
            min={1}
            max={10000}
            style={styles.input}
          />
        </label>
        <button onClick={getNewHand} disabled={loading} style={{ ...styles.button, opacity: loading ? 0.6 : 1 }}>
          {loading ? "Simulating…" : "Get random hand"}
        </button>
      </section>

      {error && <div style={styles.error}>{error}</div>}

      {currentHand && actualPercentile !== null && (
        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Guess the percentile</h2>
          <p style={styles.hint}>0% = best hand, 100% = worst. Where does this hand rank?</p>
          <div style={styles.handRow}>
            {currentHand.map((c, i) => (
              <CardDisplay key={i} card={c} size="medium" />
            ))}
          </div>
          {feedback === null ? (
            <div style={styles.guessRow}>
              <input
                type="number"
                min={0}
                max={100}
                step={0.1}
                value={guess}
                onChange={(e) => setGuess(e.target.value)}
                placeholder="0–100"
                style={styles.input}
              />
              <button onClick={submitGuess} disabled={!guess.trim()} style={styles.button}>
                Submit
              </button>
            </div>
          ) : (
            <div
              style={{
                ...styles.feedback,
                background:
                  feedback === "exact"
                    ? theme.feedbackCorrect
                    : feedback === "close"
                      ? theme.feedbackClose
                      : theme.feedbackIncorrect,
              }}
            >
              {feedback === "exact" && "Correct! (within 4%)"}
              {feedback === "close" && "Close (4–12% off)"}
              {feedback === "off" && "Off (> 12%)"}
              <span style={styles.actual}> Actual: {actualPercentile.toFixed(1)}%</span>
              <button onClick={getNewHand} style={styles.nextButton}>
                Next hand
              </button>
            </div>
          )}
        </section>
      )}

      <section style={styles.stats}>
        <h2 style={styles.sectionTitle}>Session stats</h2>
        <div style={styles.statsRow}>
          <span>Attempts: {attempts}</span>
          <span>Accuracy: {accuracy}%</span>
          <span>Streak: {streak}</span>
        </div>
      </section>
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
  handRow: { display: "flex", gap: 6, marginBottom: "1rem" },
  guessRow: { display: "flex", gap: 8, alignItems: "center" },
  feedback: {
    padding: "1rem",
    borderRadius: 6,
    color: "white",
    fontWeight: 600,
  },
  actual: { opacity: 0.9, marginLeft: 8 },
  nextButton: {
    marginLeft: 16,
    padding: "0.25rem 0.75rem",
    background: "rgba(255,255,255,0.3)",
    border: "none",
    borderRadius: 4,
    color: "white",
    cursor: "pointer",
  },
  stats: { marginTop: "2rem" },
  statsRow: { display: "flex", gap: "2rem", color: theme.textMuted },
};
