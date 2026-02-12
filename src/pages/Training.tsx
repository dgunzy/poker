import { useState, useEffect, useCallback } from "react";
import { getPreferences, setPreferences, getGameTrainingInfo } from "../api";
import { useGame } from "../context/GameContext";
import { PercentileGuessing } from "./training/PercentileGuessing";
import { HandGenerator } from "./training/HandGenerator";
import { theme } from "../theme";

type TrainingMode = "percentile" | "generator";

export function Training() {
  const { gameId } = useGame();
  const [mode, setModeState] = useState<TrainingMode>("percentile");
  const [gameHelp, setGameHelp] = useState("");

  useEffect(() => {
    getPreferences()
      .then((p) => {
        if (p.training_mode === "generator") setModeState("generator");
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    getGameTrainingInfo(gameId)
      .then((info) => setGameHelp(info.game_help))
      .catch(() => {});
  }, [gameId]);

  const setMode = useCallback((m: TrainingMode) => {
    setModeState(m);
    getPreferences()
      .then((p) => setPreferences({ ...p, training_mode: m }))
      .catch(() => {});
  }, []);

  const subtitle =
    mode === "percentile"
      ? gameHelp || "Guess the percentile \u2014 we pick a random hand, you estimate where it ranks."
      : "Hand generator \u2014 generate random hands with nice graphics for flash cards and study.";

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.title}>Training</h1>
        <p style={styles.subtitle}>{subtitle}</p>
        <div style={styles.modeRow}>
          <button
            onClick={() => setMode("percentile")}
            style={{
              ...styles.modeButton,
              ...(mode === "percentile" ? styles.modeButtonActive : {}),
            }}
          >
            Percentile guessing
          </button>
          <button
            onClick={() => setMode("generator")}
            style={{
              ...styles.modeButton,
              ...(mode === "generator" ? styles.modeButtonActive : {}),
            }}
          >
            Hand generator
          </button>
        </div>
      </header>

      <main style={styles.main}>
        {mode === "percentile" ? <PercentileGuessing /> : <HandGenerator />}
      </main>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: { minHeight: "100%" },
  header: {
    padding: "1.5rem 2rem",
    borderBottom: `1px solid ${theme.border}`,
  },
  title: { margin: 0, fontSize: "1.25rem", fontWeight: 600 },
  subtitle: {
    margin: "0.25rem 0 0.75rem",
    fontSize: "0.875rem",
    color: theme.textMuted,
  },
  modeRow: { display: "flex", gap: 8 },
  modeButton: {
    padding: "0.4rem 0.75rem",
    borderRadius: 6,
    border: `1px solid ${theme.border}`,
    background: theme.surface,
    color: theme.textMuted,
    fontSize: "0.875rem",
    cursor: "pointer",
  },
  modeButtonActive: {
    background: theme.primary,
    color: "white",
    borderColor: theme.primary,
  },
  main: { padding: "2rem", maxWidth: 700 },
};
