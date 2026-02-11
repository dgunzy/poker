import { useState, useEffect } from "react";
import { getUiConfig, getTrainingConfig, getPreferences, tauriAvailable } from "../api";
import { theme } from "../theme";

export function Settings() {
  const [preferences, setPreferencesState] = useState<{
    game_id: string;
    draw_count: number;
    training_mode: string;
    training_draw_count: number;
  } | null>(null);
  const [uiConfig, setUiConfig] = useState<{
    suits: Record<string, string>;
    feedback: Record<string, string>;
  } | null>(null);
  const [trainingConfig, setTrainingConfig] = useState<{
    exact_threshold: number;
    close_threshold: number;
    default_draw_count: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([getUiConfig(), getTrainingConfig(), getPreferences()])
      .then(([ui, train, prefs]) => {
        setUiConfig(ui);
        setTrainingConfig(train);
        setPreferencesState(prefs);
      })
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return (
      <div style={styles.container}>
        <header style={styles.header}>
          <h1 style={styles.title}>Settings</h1>
        </header>
        <main style={styles.main}>
          <div style={styles.error}>{error}</div>
        </main>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.title}>Settings</h1>
        <p style={styles.subtitle}>Configuration loaded from config files</p>
      </header>

      <main style={styles.main}>
        {preferences && tauriAvailable && (
          <section style={styles.section}>
            <h2 style={styles.sectionTitle}>Persisted preferences</h2>
            <p style={styles.hint}>Last used values (saved locally)</p>
            <div style={styles.thresholdList}>
              <div style={styles.thresholdRow}>
                <span>Game</span>
                <strong>{preferences.game_id}</strong>
              </div>
              <div style={styles.thresholdRow}>
                <span>Simulation draw count</span>
                <strong>{preferences.draw_count.toLocaleString()}</strong>
              </div>
              <div style={styles.thresholdRow}>
                <span>Training mode</span>
                <strong>{preferences.training_mode}</strong>
              </div>
              <div style={styles.thresholdRow}>
                <span>Training draw count</span>
                <strong>{preferences.training_draw_count.toLocaleString()}</strong>
              </div>
            </div>
          </section>
        )}

        {!tauriAvailable && (
          <div style={styles.banner}>
            Running in browser. Run <code>npm run tauri dev</code> for full config from config files.
          </div>
        )}

        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Suit colors</h2>
          <p style={styles.hint}>Card background colors by suit (from config/ui.toml)</p>
          <div style={styles.colorGrid}>
            {(uiConfig?.suits ? Object.entries(uiConfig.suits) : Object.entries(theme.suits)).map(
              ([name, color]) => (
                <div key={name} style={styles.colorRow}>
                  <div style={{ ...styles.colorSwatch, background: color }} />
                  <span style={styles.colorLabel}>{name}</span>
                  <code style={styles.colorCode}>{color}</code>
                </div>
              )
            )}
          </div>
        </section>

        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Feedback colors</h2>
          <p style={styles.hint}>Training feedback (from config/ui.toml)</p>
          <div style={styles.colorGrid}>
            {(uiConfig?.feedback ? Object.entries(uiConfig.feedback) : Object.entries(theme.feedback)).map(
              ([name, color]) => (
                <div key={name} style={styles.colorRow}>
                  <div style={{ ...styles.colorSwatch, background: color }} />
                  <span style={styles.colorLabel}>{name}</span>
                  <code style={styles.colorCode}>{color}</code>
                </div>
              )
            )}
          </div>
        </section>

        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Training thresholds</h2>
          <p style={styles.hint}>Percentile guessing game (from config/training.toml)</p>
          <div style={styles.thresholdList}>
            <div style={styles.thresholdRow}>
              <span>Exact (within)</span>
              <strong>{trainingConfig?.exact_threshold ?? 4}%</strong>
            </div>
            <div style={styles.thresholdRow}>
              <span>Close (within)</span>
              <strong>{trainingConfig?.close_threshold ?? 12}%</strong>
            </div>
            <div style={styles.thresholdRow}>
              <span>Default draw count</span>
              <strong>{trainingConfig?.default_draw_count ?? 1000}</strong>
            </div>
          </div>
        </section>
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
  subtitle: { margin: "0.25rem 0 0", fontSize: "0.875rem", color: theme.textMuted },
  main: { padding: "2rem", maxWidth: 600 },
  banner: {
    padding: "0.75rem",
    background: theme.surface,
    border: `1px solid ${theme.border}`,
    borderRadius: 6,
    marginBottom: "1.5rem",
    fontSize: "0.875rem",
  },
  section: { marginBottom: "2rem" },
  sectionTitle: { margin: "0 0 0.5rem", fontSize: "1rem", fontWeight: 600 },
  hint: { margin: "0 0 0.75rem", fontSize: "0.8rem", color: theme.textMuted },
  colorGrid: { display: "flex", flexDirection: "column", gap: 8 },
  colorRow: { display: "flex", alignItems: "center", gap: 12 },
  colorSwatch: { width: 24, height: 24, borderRadius: 4 },
  colorLabel: { textTransform: "capitalize", minWidth: 80 },
  colorCode: { fontSize: "0.8rem", color: theme.textMuted },
  thresholdList: { display: "flex", flexDirection: "column", gap: 8 },
  thresholdRow: { display: "flex", justifyContent: "space-between" },
  error: {
    padding: "0.75rem",
    background: theme.feedbackIncorrect,
    color: "white",
    borderRadius: 6,
  },
};
