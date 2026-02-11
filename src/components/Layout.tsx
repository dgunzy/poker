import { Link, Outlet, useLocation } from "react-router-dom";
import { theme } from "../theme";
import { GameSelector } from "./GameSelector";
import { useGame } from "../context/GameContext";

export function Layout() {
  const location = useLocation();
  const { gameId, setGameId } = useGame();

  const navItems = [
    { path: "/", label: "Home" },
    { path: "/simulation", label: "Simulation" },
    { path: "/training", label: "Training" },
    { path: "/settings", label: "Settings" },
  ];

  return (
    <div style={styles.container}>
      <aside style={styles.sidebar}>
        <div style={styles.logo}>Poker</div>
        <div style={styles.gameRow}>
          <GameSelector value={gameId} onChange={setGameId} />
        </div>
        <nav style={styles.nav}>
          {navItems.map(({ path, label }) => {
            const active = location.pathname === path || (path !== "/" && location.pathname.startsWith(path));
            return (
              <Link
                key={path}
                to={path}
                className="nav-link"
                style={{
                  ...styles.navItem,
                  ...(active ? styles.navItemActive : {}),
                }}
              >
                {label}
              </Link>
            );
          })}
        </nav>
      </aside>
      <div style={styles.main}>
        <Outlet />
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    minHeight: "100vh",
    background: theme.background,
    color: theme.text,
    fontFamily: theme.fontFamily,
  },
  sidebar: {
    width: theme.sidebarWidth,
    borderRight: `1px solid ${theme.border}`,
    padding: "1rem 0",
  },
  logo: {
    padding: "0 1.25rem",
    fontSize: "1.25rem",
    fontWeight: 700,
    marginBottom: "0.5rem",
  },
  gameRow: {
    padding: "0 1.25rem",
    marginBottom: "1rem",
    minWidth: 0,
    overflow: "hidden",
  },
  nav: {
    display: "flex",
    flexDirection: "column",
    gap: 2,
  },
  navItem: {
    padding: "0.5rem 1.25rem",
    color: theme.textMuted,
    textDecoration: "none",
    fontSize: "0.9rem",
  },
  navItemActive: {
    color: theme.primary,
    background: `${theme.primary}20`,
    borderRight: `3px solid ${theme.primary}`,
  },
  main: {
    flex: 1,
    overflow: "auto",
  },
};
