import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from "react";
import { getPreferences, setPreferences } from "../api";

const GameContext = createContext<{ gameId: string; setGameId: (id: string) => void }>({
  gameId: "deuce_to_seven",
  setGameId: () => {},
});

export function GameProvider({ children }: { children: ReactNode }) {
  const [gameId, setGameIdState] = useState("deuce_to_seven");
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    getPreferences()
      .then((p) => {
        if (p.game_id) setGameIdState(p.game_id);
      })
      .catch(() => {})
      .finally(() => setLoaded(true));
  }, []);

  const setGameId = useCallback((id: string) => {
    setGameIdState(id);
    getPreferences()
      .then((p) => setPreferences({ ...p, game_id: id }))
      .catch(() => {});
  }, []);

  return (
    <GameContext.Provider value={{ gameId, setGameId }}>
      {children}
    </GameContext.Provider>
  );
}

export function useGame() {
  return useContext(GameContext);
}
