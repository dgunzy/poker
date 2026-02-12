import { useState, useEffect } from "react";
import {
  getTrainingConfig,
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

type Feedback = "exact" | "close" | "off" | null;

export function PercentileGuessing() {
  const { gameId } = useGame();
  const [trainingInfo, setTrainingInfo] = useState<GameTrainingInfo | null>(null);
  const [heldCards, setHeldCards] = useState<CardSelection[]>([]);
  const [deadCards, setDeadCards] = useState<CardSelection[]>([]);
  const [drawCount, setDrawCount] = useState(1000);
  const [exactThreshold, setExactThreshold] = useState(4);
  const [closeThreshold, setCloseThreshold] = useState(12);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [scenario, setScenario] = useState<TrainingScenario | null>(null);
  const [guess, setGuess] = useState("");
  const [lowGuess, setLowGuess] = useState("");
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [lowFeedback, setLowFeedback] = useState<Feedback>(null);
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

  useEffect(() => {
    getGameTrainingInfo(gameId).then(setTrainingInfo).catch(() => {});
    setHeldCards([]);
    setDeadCards([]);
    setScenario(null);
    setFeedback(null);
    setLowFeedback(null);
  }, [gameId]);

  const heldMaxCards = trainingInfo?.max_held ?? 5;

  const getNewHand = async () => {
    setLoading(true);
    setError(null);
    setScenario(null);
    setGuess("");
    setLowGuess("");
    setFeedback(null);
    setLowFeedback(null);
    try {
      getPreferences().then((p) => setPreferences({ ...p, training_draw_count: drawCount })).catch(() => {});
      const result = await generateTrainingScenario({
        held_cards: heldCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        dead_cards: deadCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        sim_count: drawCount,
        game_id: gameId,
      });
      setScenario(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const getActualPercentile = (): number | null => {
    if (!scenario) return null;
    if (scenario.type === "draw") return scenario.percentile;
    if (scenario.type === "board") return scenario.percentile;
    if (scenario.type === "split_pot") return scenario.high_percentile;
    return null;
  };

  const submitGuess = () => {
    const g = parseFloat(guess);
    const actual = getActualPercentile();
    if (isNaN(g) || actual === null) return;

    // For split_pot, also evaluate low guess if low qualifies.
    if (scenario?.type === "split_pot" && scenario.low_qualifies && scenario.low_percentile !== null) {
      const lg = parseFloat(lowGuess);
      if (isNaN(lg)) return;
      const lowDiff = Math.abs(lg - scenario.low_percentile);
      const lowFb: Feedback = lowDiff <= exactThreshold ? "exact" : lowDiff <= closeThreshold ? "close" : "off";
      setLowFeedback(lowFb);
    }

    setAttempts((a) => a + 1);
    const diff = Math.abs(g - actual);
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

  const accuracy = attempts > 0 ? ((correct / attempts) * 100).toFixed(0) : "\u2014";
  const displayMode = trainingInfo?.display_mode ?? "draw";

  const renderHandDisplay = () => {
    if (!scenario) return null;

    if (scenario.type === "draw") {
      return (
        <div style={styles.handRow}>
          {scenario.hand.map((c, i) => (
            <CardDisplay key={i} card={c} size="medium" />
          ))}
        </div>
      );
    }

    const holeCards = scenario.type === "board" ? scenario.hole_cards : scenario.hole_cards;
    const board = scenario.type === "board" ? scenario.board : scenario.board;

    return (
      <div>
        <h3 style={styles.subTitle}>Hole cards</h3>
        <div style={styles.handRow}>
          {holeCards.map((c, i) => (
            <CardDisplay key={`hole-${i}`} card={c} size="medium" />
          ))}
        </div>
        <h3 style={{ ...styles.subTitle, marginTop: "0.75rem" }}>Board</h3>
        <div style={styles.handRow}>
          {board.map((c, i) => (
            <CardDisplay key={`board-${i}`} card={c} size="medium" />
          ))}
        </div>
      </div>
    );
  };

  const renderGuessSection = () => {
    if (!scenario) return null;
    const isSplit = scenario.type === "split_pot";
    const showLowInput = isSplit && scenario.low_qualifies;
    const canSubmit = guess.trim() && (!showLowInput || lowGuess.trim());

    return (
      <div>
        <div style={styles.guessRow}>
          <label style={styles.guessLabel}>
            {isSplit ? "High percentile:" : "Percentile:"}
            <input
              type="number"
              min={0}
              max={100}
              step={0.1}
              value={guess}
              onChange={(e) => setGuess(e.target.value)}
              placeholder="0\u2013100"
              style={styles.input}
            />
          </label>
        </div>
        {isSplit && !scenario.low_qualifies && (
          <div style={styles.noLowBadge}>No qualifying low</div>
        )}
        {showLowInput && (
          <div style={styles.guessRow}>
            <label style={styles.guessLabel}>
              Low percentile:
              <input
                type="number"
                min={0}
                max={100}
                step={0.1}
                value={lowGuess}
                onChange={(e) => setLowGuess(e.target.value)}
                placeholder="0\u2013100"
                style={styles.input}
              />
            </label>
          </div>
        )}
        <button onClick={submitGuess} disabled={!canSubmit} style={styles.button}>
          Submit
        </button>
      </div>
    );
  };

  const renderFeedback = () => {
    if (!scenario || feedback === null) return null;

    const actual = getActualPercentile();
    const isSplit = scenario.type === "split_pot";

    const fbColor = feedback === "exact" ? theme.feedbackCorrect : feedback === "close" ? theme.feedbackClose : theme.feedbackIncorrect;
    const fbText = feedback === "exact" ? "Correct! (within 4%)" : feedback === "close" ? "Close (4\u201312% off)" : "Off (> 12%)";

    return (
      <div>
        <div style={{ ...styles.feedback, background: fbColor }}>
          {isSplit ? "High: " : ""}{fbText}
          <span style={styles.actual}> Actual: {actual?.toFixed(1)}%</span>
        </div>
        {isSplit && scenario.type === "split_pot" && scenario.low_qualifies && lowFeedback !== null && (
          <div
            style={{
              ...styles.feedback,
              marginTop: 8,
              background:
                lowFeedback === "exact" ? theme.feedbackCorrect : lowFeedback === "close" ? theme.feedbackClose : theme.feedbackIncorrect,
            }}
          >
            Low: {lowFeedback === "exact" ? "Correct!" : lowFeedback === "close" ? "Close" : "Off"}
            <span style={styles.actual}> Actual: {scenario.low_percentile?.toFixed(1)}%</span>
          </div>
        )}
        {isSplit && scenario.type === "split_pot" && !scenario.low_qualifies && (
          <div style={{ ...styles.noLowBadge, marginTop: 8 }}>No qualifying low</div>
        )}
        <button onClick={getNewHand} style={{ ...styles.nextButton, marginTop: 12 }}>
          Next hand
        </button>
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
          {displayMode === "draw" ? "Simulations (100 or 1,000 typical)" : "Simulations"}:
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
          {loading ? "Simulating\u2026" : "Get random hand"}
        </button>
      </section>

      {error && <div style={styles.error}>{error}</div>}

      {scenario && (
        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Guess the percentile</h2>
          <p style={styles.hint}>0% = best hand, 100% = worst. Where does this hand rank?</p>
          {renderHandDisplay()}
          {feedback === null ? renderGuessSection() : renderFeedback()}
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
  guessRow: { display: "flex", gap: 8, alignItems: "center", marginBottom: "0.5rem" },
  guessLabel: { display: "flex", alignItems: "center", gap: "0.5rem", fontSize: "0.9rem" },
  feedback: {
    padding: "1rem",
    borderRadius: 6,
    color: "white",
    fontWeight: 600,
  },
  actual: { opacity: 0.9, marginLeft: 8 },
  nextButton: {
    padding: "0.25rem 0.75rem",
    background: "rgba(255,255,255,0.3)",
    border: "none",
    borderRadius: 4,
    color: "white",
    cursor: "pointer",
  },
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
  stats: { marginTop: "2rem" },
  statsRow: { display: "flex", gap: "2rem", color: theme.textMuted },
};
