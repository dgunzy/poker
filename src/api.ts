/**
 * Safe Tauri API layer. Handles missing Tauri context (e.g. running in browser dev).
 */

function isTauriAvailable(): boolean {
  if (typeof window === "undefined") return false;
  const internals = (window as unknown as { __TAURI_INTERNALS__?: { invoke?: unknown } }).__TAURI_INTERNALS__;
  return typeof internals?.invoke === "function";
}

export async function runSimulation(input: {
  held_cards: Array<{ rank: string; suit: string }>;
  dead_cards: Array<{ rank: string; suit: string }>;
  draw_count: number;
  seed?: number;
  game_id?: string;
}): Promise<{
  results: Array<{ hand: Array<{ rank: string; suit: string }>; rank: number; percentile: number }>;
  draw_count: number;
  best_rank: number;
  worst_rank: number;
  seed_used: number;
}> {
  if (!isTauriAvailable()) {
    throw new Error("Tauri API not available. Run 'npm run tauri dev' to use simulation.");
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("run_simulation", { input });
}

export async function getTrainingConfig(): Promise<{
  exact_threshold: number;
  close_threshold: number;
  default_draw_count: number;
}> {
  if (!isTauriAvailable()) {
    return { exact_threshold: 4, close_threshold: 12, default_draw_count: 1000 };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_training_config");
}

export async function getUiConfig(): Promise<{
  suits: Record<string, string>;
  feedback: Record<string, string>;
}> {
  if (!isTauriAvailable()) {
    return {
      suits: { clubs: "#2d5a27", diamonds: "#1e3a8a", hearts: "#991b1b", spades: "#374151" },
      feedback: { correct: "#22c55e", close: "#eab308", incorrect: "#ef4444" },
    };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_ui_config");
}

export async function getGames(): Promise<
  Array<{ id: string; name: string; hand_size: number; deck_size: number; description: string }>
> {
  if (!isTauriAvailable()) {
    return [
      { id: "deuce_to_seven", name: "NL 2-7", hand_size: 5, deck_size: 52, description: "Deuce-to-Seven Lowball" },
      { id: "ace_to_five", name: "A-5 / Razz", hand_size: 5, deck_size: 52, description: "Ace-to-Five lowball" },
      { id: "holdem", name: "NLHE", hand_size: 2, deck_size: 52, description: "No Limit Hold'em" },
      { id: "omaha", name: "PLO", hand_size: 4, deck_size: 52, description: "Pot Limit Omaha" },
      { id: "omaha8", name: "O8", hand_size: 4, deck_size: 52, description: "Omaha Hi-Lo 8-or-better" },
    ];
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_games");
}

export interface Preferences {
  game_id: string;
  draw_count: number;
  training_mode: string;
  training_draw_count: number;
}

export async function getPreferences(): Promise<Preferences> {
  if (!isTauriAvailable()) {
    return {
      game_id: "deuce_to_seven",
      draw_count: 1000,
      training_mode: "percentile",
      training_draw_count: 1000,
    };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_preferences");
}

export async function setPreferences(prefs: Preferences): Promise<void> {
  if (!isTauriAvailable()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_preferences", { prefs });
}

export const tauriAvailable = isTauriAvailable();
