import { useState, useEffect } from "react";
import { runSimulation as runSimulationApi, getPreferences, setPreferences } from "../api";
import { useGame } from "../context/GameContext";
import { CardPicker } from "../components/CardPicker";
import { CardDisplay } from "../components/CardDisplay";
import { theme } from "../theme";

export interface CardSelection {
  rank: string;
  suit: string;
  index: number;
}

function cardToSelection(card: { rank: string; suit: string }, index: number): CardSelection {
  return { rank: card.rank, suit: card.suit, index };
}

type SortKey = "rank" | "percentile" | "none";

export function Simulation() {
  const { gameId } = useGame();
  const heldMaxCards = gameId === "holdem" ? 2 : gameId === "omaha" || gameId === "omaha8" ? 4 : 5;
  const isDrawGame = gameId === "deuce_to_seven" || gameId === "ace_to_five";
  const heldLabel = isDrawGame ? "Your draw" : "Held cards";
  const [heldCards, setHeldCards] = useState<CardSelection[]>([]);
  const [deadCards, setDeadCards] = useState<CardSelection[]>([]);
  const [drawCount, setDrawCount] = useState(1000);

  useEffect(() => {
    getPreferences()
      .then((p) => {
        if (p.draw_count >= 1) setDrawCount(p.draw_count);
      })
      .catch(() => {});
  }, []);
  const [results, setResults] = useState<Array<{ hand: CardSelection[]; rank: number; percentile: number }> | null>(null);
  const [summary, setSummary] = useState<{ best_rank: number; worst_rank: number; draw_count: number } | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>("none");
  const [sortAsc, setSortAsc] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runSimulation = async () => {
    setLoading(true);
    setError(null);
    setResults(null);
    setSummary(null);
    try {
      getPreferences()
        .then((p) => setPreferences({ ...p, draw_count: drawCount }))
        .catch(() => {});
      const input = {
        held_cards: heldCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        dead_cards: deadCards.map((c) => ({ rank: c.rank, suit: c.suit })),
        draw_count: drawCount,
        game_id: gameId,
      };
      const res = await runSimulationApi(input);
      setSummary({ best_rank: res.best_rank, worst_rank: res.worst_rank, draw_count: res.draw_count });
      setResults(
        res.results.map((r) => ({
          hand: r.hand.map((c, i) => cardToSelection(c, i)),
          rank: r.rank,
          percentile: r.percentile,
        }))
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortAsc((a) => !a);
    } else {
      setSortKey(key);
      setSortAsc(true);
    }
  };

  const sortedResults = results
    ? [...results].sort((a, b) => {
        if (sortKey === "none") return 0;
        const mult = sortAsc ? 1 : -1;
        if (sortKey === "rank") return mult * (a.rank - b.rank);
        if (sortKey === "percentile") return mult * (a.percentile - b.percentile);
        return 0;
      })
    : null;

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.title}>Simulation</h1>
        <p style={styles.subtitle}>Monte Carlo draw simulation</p>
      </header>

      <main style={styles.main}>
        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>{heldLabel}</h2>
          <p style={styles.hint}>
            {isDrawGame
              ? "Cards you're keeping (e.g. 8763 for a 1-card draw). Dead cards = folded/seen. Click + to add."
              : `0–${heldMaxCards} hole cards. We pad with random and simulate boards. Dead cards excluded.`}
          </p>
          <CardPicker
            selected={heldCards}
            onChange={setHeldCards}
            maxCards={heldMaxCards}
            excludedCards={deadCards}
          />
        </section>

        <section style={styles.section}>
          <h2 style={styles.sectionTitle}>Dead Cards</h2>
          <p style={styles.hint}>Cards seen or out of play (opponent mucks, etc.)</p>
          <CardPicker
            selected={deadCards}
            onChange={setDeadCards}
            maxCards={20}
            excludedCards={heldCards}
          />
        </section>

        <section style={styles.section}>
          <label style={styles.label}>
            Draw count:
            <input
              type="number"
              value={drawCount}
              onChange={(e) => setDrawCount(Math.max(1, Math.min(100000, parseInt(e.target.value) || 1000)))}
              min={1}
              max={100000}
              step={100}
              style={styles.input}
            />
          </label>
          <button
            onClick={runSimulation}
            disabled={loading}
            style={{
              ...styles.button,
              opacity: loading ? 0.6 : 1,
            }}
          >
            {loading ? "Running..." : "Run Simulation"}
          </button>
        </section>

        {error && (
          <div style={styles.error}>{error}</div>
        )}

        {summary && (
          <section style={styles.summary}>
            <h2 style={styles.sectionTitle}>Summary</h2>
            <div style={styles.summaryRow}>
              <span>Draws: {summary.draw_count.toLocaleString()}</span>
              <span>Best rank: {summary.best_rank}</span>
              <span>Worst rank: {summary.worst_rank}</span>
            </div>
          </section>
        )}

        {sortedResults && (
          <section style={styles.results}>
            <h2 style={styles.sectionTitle}>Results</h2>
            <div style={styles.resultsTable}>
              <div style={styles.row}>
                <button
                  style={styles.colHeader}
                  onClick={() => toggleSort("rank")}
                >
                  Rank {sortKey === "rank" && (sortAsc ? "↑" : "↓")}
                </button>
                <span style={styles.colHeader}>Hand</span>
                <button
                  style={styles.colHeader}
                  onClick={() => toggleSort("percentile")}
                >
                  Percentile {sortKey === "percentile" && (sortAsc ? "↑" : "↓")}
                </button>
              </div>
              {sortedResults.slice(0, 50).map((r, i) => (
                <div key={i} style={styles.row}>
                  <span>{i + 1}</span>
                  <div style={styles.handRow}>
                    {r.hand.map((c, j) => (
                      <CardDisplay key={j} card={c} size="xs" />
                    ))}
                  </div>
                  <span>{r.percentile.toFixed(1)}%</span>
                </div>
              ))}
            </div>
            {sortedResults.length > 50 && (
              <p style={styles.more}>... and {sortedResults.length - 50} more</p>
            )}
          </section>
        )}
      </main>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: "100%",
  },
  header: {
    padding: "1.5rem 2rem",
    borderBottom: `1px solid ${theme.border}`,
  },
  title: {
    margin: 0,
    fontSize: "1.25rem",
    fontWeight: 600,
  },
  subtitle: {
    margin: "0.25rem 0 0",
    fontSize: "0.875rem",
    color: theme.textMuted,
  },
  main: {
    padding: "2rem",
    maxWidth: 900,
    margin: "0 auto",
  },
  section: {
    marginBottom: "2rem",
  },
  sectionTitle: {
    margin: "0 0 0.5rem",
    fontSize: "1rem",
    fontWeight: 600,
  },
  hint: {
    margin: "0 0 0.75rem",
    fontSize: "0.8rem",
    color: theme.textMuted,
  },
  label: {
    display: "flex",
    alignItems: "center",
    gap: "0.5rem",
    marginBottom: "0.75rem",
  },
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
  results: {
    marginTop: "2rem",
  },
  resultsTable: {
    border: `1px solid ${theme.border}`,
    borderRadius: 8,
    overflow: "hidden",
  },
  row: {
    display: "grid",
    gridTemplateColumns: "60px 1fr 80px",
    gap: "1rem",
    padding: "0.5rem 1rem",
    alignItems: "center",
    borderBottom: `1px solid ${theme.border}`,
  },
  summary: { marginBottom: "1.5rem" },
  summaryRow: {
    display: "flex",
    gap: "2rem",
    color: theme.textMuted,
    fontSize: "0.9rem",
  },
  colHeader: {
    fontWeight: 600,
    background: "none",
    border: "none",
    color: "inherit",
    cursor: "pointer",
    padding: 0,
    textAlign: "left",
    font: "inherit",
  },
  handRow: {
    display: "flex",
    gap: "0.25rem",
  },
  more: {
    marginTop: "0.5rem",
    color: theme.textMuted,
    fontSize: "0.9rem",
  },
};
