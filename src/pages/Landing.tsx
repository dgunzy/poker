import { Link } from "react-router-dom";
import { theme } from "../theme";

export function Landing() {
  return (
    <div style={styles.container}>
      <div style={styles.hero}>
        <h1 style={styles.title}>Poker Simulator</h1>
        <p style={styles.subtitle}>
          Train and analyze NL 2-7 Single Draw. Run Monte Carlo simulations,
          evaluate hand strength, and improve your lowball instincts.
        </p>
      </div>

      <div style={styles.cards}>
        <Link to="/simulation" className="landing-card" style={styles.card}>
          <div style={styles.cardIcon}>📊</div>
          <h2 style={styles.cardTitle}>Simulation</h2>
          <p style={styles.cardDesc}>
            Run draw simulations with held and dead cards. See percentile ranks
            and hand distributions.
          </p>
        </Link>

        <Link to="/training" className="landing-card" style={styles.card}>
          <div style={styles.cardIcon}>🎯</div>
          <h2 style={styles.cardTitle}>Training</h2>
          <p style={styles.cardDesc}>
            Percentile guessing game. Practice estimating hand strength and
            track your accuracy.
          </p>
        </Link>

        <Link to="/settings" className="landing-card" style={styles.card}>
          <div style={styles.cardIcon}>⚙️</div>
          <h2 style={styles.cardTitle}>Settings</h2>
          <p style={styles.cardDesc}>
            Game preferences, scoring thresholds, and theme options.
          </p>
        </Link>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: "3rem 2rem",
    maxWidth: 800,
    margin: "0 auto",
  },
  hero: {
    marginBottom: "3rem",
  },
  title: {
    margin: 0,
    fontSize: "2rem",
    fontWeight: 600,
  },
  subtitle: {
    margin: "0.75rem 0 0",
    fontSize: "1rem",
    color: theme.textMuted,
    lineHeight: 1.5,
  },
  cards: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
    gap: "1rem",
  },
  card: {
    padding: "1.5rem",
    background: theme.surface,
    border: `1px solid ${theme.border}`,
    borderRadius: 8,
    textDecoration: "none",
    color: "inherit",
    transition: "border-color 0.15s, background 0.15s",
  },
  cardIcon: {
    fontSize: "2rem",
    marginBottom: "0.75rem",
  },
  cardTitle: {
    margin: 0,
    fontSize: "1.1rem",
    fontWeight: 600,
  },
  cardDesc: {
    margin: "0.5rem 0 0",
    fontSize: "0.875rem",
    color: theme.textMuted,
    lineHeight: 1.4,
  },
};
