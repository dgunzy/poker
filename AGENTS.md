# Poker Simulator — Architecture Document

## Project Overview

A cross-platform, native poker simulation and training application built in Rust. The application provides draw simulation, hand evaluation, percentile analysis, and training tools for lowball and draw poker variants. Designed for speed, extensibility, and a clean native user experience.

---

## Client Requirements (Source of Truth)

**Core simulator (NL 2-7 Single Draw):**
- Input: your draw (held cards), dead cards, number of simulations (100 or 1,000)
- Simulate the draw N times; each run completes your hand by drawing from the remaining deck
- Output: table with columns **Rank | Hand | Percentile**, sorted by rank
- Percentile: 0 = best hand, 100 = worst. With 1,000 sims → 0.1/0.2/0.3% increments; with 100 → 1/2/3%
- Support any number of dead cards (e.g. opponent's draw vs your dead cards)

**Example:** Start with 8763K. Input draw = 8763, dead = K. Best possible draw = 87632, worst = 87638 (unless flush). Rules for straights (e.g. 8764) — inverse poker rankings.

**Training tools (two modes):**
1. **Percentile guessing** — Pick one random hand from the simulated results, ask user to guess its percentile. Score: Green (within 4%), Yellow (4–12%), Red (> 12%). Track session stats.
2. **Hand generator** — Generate random hands with nice graphics for flash cards and study. Setup optional for draw games (0 held = pure random); required for board games. Shows hand + percentile. "Next" for another.

**Future:** Hand notation input (e.g. type "KsQsJs4h" and render pretty cards).

---

### Supported Games (Current)

- NL 2-7 Single Draw, A-5/Razz, NLHE, PLO, O8
- Badugi (planned)

### Target Platforms

- Windows (primary)
- macOS (primary)
- Linux (future, minimal additional effort)

---

## Implementation Status (Current)

### Completed

- **poker-core** — Card, Suit, Rank, Deck, Hand types ✓
- **poker-eval** — Evaluator trait, DeuceToSevenEvaluator (reference implementation) ✓
- **poker-sim** — Monte Carlo simulation with Rayon, held/dead cards, percentile results ✓
- **poker-app (src-tauri)** — Tauri 2 shell, `run_simulation`, `get_games`, `get_training_config`, `get_ui_config` ✓
- **Game selector** — Sidebar shows active game (from config/games.toml); ready for multi-game ✓
- **Frontend** — React + TypeScript, routing, landing page, simulation page ✓
- **UI** — Sidebar layout, card picker with suit color-coding, reusable CardDisplay ✓
- **API layer** — Safe Tauri invoke wrapper (`src/api.ts`); checks `__TAURI_INTERNALS__` before calling ✓
- **Training** — Percentile guessing game: setup (held/dead cards, draw count), get random hand, submit guess, exact/close/off feedback, session stats ✓
- **Settings** — Display config from `config/ui.toml` and `config/training.toml` (suit colors, feedback colors, training thresholds) ✓
- **Simulation summary** — Draw count, best/worst rank above results ✓
- **Sortable results** — Click Rank or Percentile column headers to sort ✓

### Planned

- **poker-training** crate — Training session persistence, storage trait
- **Lookup table evaluator** — Replace reference evaluator with table-based (tools/table-gen)
- **Settings editing** — Save user preferences to config

---

## Design Principles

- **Idiomatic Rust** — leverage the type system, traits, enums, and ownership model to enforce correctness at compile time. Prefer zero-cost abstractions. No runtime polymorphism where static dispatch suffices.
- **Extensibility through traits** — new poker variants are added by implementing core traits, not by modifying existing code. The engine should be game-agnostic.
- **Constants over magic values** — all game parameters, thresholds, and configuration live in typed constants or configuration files. No string literals scattered through business logic.
- **Performance-first evaluation** — hand evaluation uses precomputed lookup tables and state machines. Simulation hot paths must be allocation-free and branch-minimal.
- **GPU-ready architecture** — the compute layer is designed so that equity calculations and large-scale simulations can be offloaded to GPU compute shaders (via wgpu) in the future without restructuring the core.
- **Separation of concerns** — the engine knows nothing about the UI. The UI knows nothing about hand evaluation internals. Communication happens through well-defined command/result types.

---

## Workspace Structure

```
poker/
├── Cargo.toml                    # Workspace root
├── config/
│   ├── games.toml                # Game definitions, parameters
│   ├── training.toml             # Scoring thresholds, session defaults
│   └── ui.toml                   # Theme colors, card rendering config
│
├── crates/
│   ├── poker-core/               # Fundamental types ✓
│   ├── poker-eval/               # Hand evaluation engines ✓
│   └── poker-sim/                # Simulation and equity engine ✓
│
├── src-tauri/                    # Tauri application shell ✓
│   ├── src/
│   │   ├── lib.rs                # IPC commands, app entry
│   │   └── main.rs
│   ├── capabilities/
│   └── tauri.conf.json
│
├── src/                          # React frontend ✓
│   ├── components/               # CardPicker, CardDisplay, Layout
│   ├── pages/                    # Landing, Simulation, Training, Settings
│   ├── theme.ts                  # Design tokens, RANK_DISPLAY
│   └── App.tsx                   # Router, routes
│
├── tables/                       # Generated lookup tables (future)
│   └── deuce_to_seven_5card.bin
│
└── tools/
    └── table-gen/                # Lookup table generator (future)
```

---

## Crate: `poker-core`

Foundational types shared across all other crates. No dependencies on evaluation or simulation logic.

### Responsibilities

- Card representation using a compact numeric encoding (0–51). Rank and suit extractable via bit operations.
- Suit and Rank as enums with exhaustive matching. No raw integers leak outside this crate.
- Deck type with support for removing dead cards and drawing from the remainder. Uses a Fisher-Yates shuffle with a pluggable RNG source.
- Hand type as a fixed-size array of cards with compile-time size validation per game variant.
- Card collection utilities — set operations, membership checks, iteration.

### Design Notes

- Cards are represented as `u8` internally for cache efficiency, but all public APIs use the `Card` newtype to prevent misuse.
- Suits use a canonical ordering for the suit-remapping optimization in the evaluator. The internal ordering is defined as a constant, not assumed.
- The `Deck` type tracks removed cards via a bitset (u64) for O(1) membership checks and efficient iteration of remaining cards.

---

## Crate: `poker-eval`

Pluggable hand evaluation engines. Each poker variant provides its own evaluator that implements a common trait.

### Core Trait

The evaluator trait defines a single responsibility: given a set of cards, produce a numeric rank where lower values represent stronger hands. Every game variant implements this trait. The trait is object-safe to allow dynamic dispatch when needed (e.g., game selection at runtime), but the simulation hot path uses monomorphized generic calls for zero-cost evaluation.

### State Machine Evaluator

The primary evaluation strategy uses a precomputed state machine (based on the 2+2 evaluator approach):

- A directed graph where each state has 52 outgoing transitions (one per card).
- Traversing a transition advances the evaluator by one card.
- Terminal states contain the final hand rank.
- Equivalent hands (after suit remapping) share states, compressing the state space dramatically.
- For 5-card evaluation: approximately 3,459 states.
- For 7-card evaluation (future): approximately 163,060 states.

#### Suit Remapping

Each transition includes a suit permutation lookup. Since hand evaluation only cares about flush potential (not which specific suit), suits are remapped to canonical forms at each step. This collapses isomorphic states and keeps the table small enough to fit in L1/L2 cache.

The remapping tables are small, separate lookup arrays stored alongside the main state table.

### Lookup Table Generation

Tables are generated by a standalone tool (`tools/table-gen/`) at development time, not at application build time. Generated tables are checked into the repository as binary files and embedded into the evaluator crate via `include_bytes!`. This keeps build times fast and ensures deterministic evaluation.

The table generator is exhaustive — it enumerates all possible card sequences, merges equivalent states, and validates the output against a reference evaluator.

### Game-Specific Evaluators

#### NL 2-7 Lowball (Deuce-to-Seven)

- Aces are always high.
- Straights and flushes count against the hand (they are bad).
- The best possible hand is 7-5-4-3-2 with at least two suits represented.
- Uses the same state machine architecture but with inverted ranking: unpaired, non-straight, non-flush hands receive the lowest (best) rank values.
- Ranking tiebreakers compare cards high-to-low (lower cards are better).

#### Badugi (Future)

- 4-card game where suits are structurally significant (one card per suit is ideal).
- Duplicate suits cause cards to be eliminated, creating 1-card, 2-card, or 3-card badugi hands.
- Requires a separate state machine with different state compression characteristics.
- The evaluator trait accommodates variable hand sizes through associated types.

#### Standard High Hand (Future)

- Traditional poker hand rankings for equity calculations and training on standard games.

---

## Crate: `poker-sim`

Simulation engine that draws cards, evaluates hands, and produces ranked results with percentile data.

### Responsibilities

- Accept a held hand, dead cards, and a game variant. Construct the remaining live deck.
- Run N simulated draws, evaluating each completed hand.
- Collect results into a sorted, ranked structure with percentile assignments.
- Support configurable draw counts (100, 1,000, 10,000+).
- Provide simulation metadata: best possible hand, worst possible hand, distribution statistics.

### Simulation Pipeline

1. **Input validation** — verify held cards and dead cards are disjoint, total cards make sense for the game variant, suits are valid.
2. **Deck construction** — build the live deck by removing held and dead cards. Store as a compact array for cache-friendly iteration.
3. **Draw loop** — for each trial, shuffle the live deck (or use partial shuffle for the number of cards needed), complete the hand, evaluate, store the rank.
4. **Result aggregation** — sort ranks, assign percentile positions, identify best/worst results, compute distribution buckets.

### Concurrency

The simulation loop is embarrassingly parallel. The engine splits N trials across available CPU cores using `rayon` for work-stealing parallelism. Each thread gets its own RNG (seeded deterministically from a master seed for reproducibility) and deck copy.

### Future: GPU Compute Path

For expensive equity calculations (range vs. range, full enumeration rather than Monte Carlo), the simulation crate defines a compute trait that can be backed by either CPU or GPU:

- CPU path: the current `rayon`-based parallel simulation.
- GPU path (future, via `poker-gpu`): batch hand data is uploaded to GPU buffers, a compute shader performs evaluation in parallel across thousands of threads, results are read back.

The simulation crate does not depend on `poker-gpu` directly. Instead, it accepts a generic compute backend through a trait, allowing the GPU crate to be an optional dependency enabled via Cargo feature flag.

### Result Types

Simulation results are structured data, not formatted strings. The result type includes:

- A sorted array of (hand, rank, percentile) tuples.
- Summary statistics (mean rank, median, standard deviation).
- The input parameters (held cards, dead cards, draw count) for reproducibility.
- A timestamp and RNG seed for exact replay if needed.

---

## Crate: `poker-training`

Training tools built on top of the simulation engine.

### Training Modes

**Percentile guessing** — primary drill. Flow:

1. **Setup:** User configures their scenario for the chosen game.
   - **Draw games (2-7, A-5):** "Your draw" = 0–5 cards you're keeping. "Dead cards" = out of play. Simulations = draws to run. We simulate random completions to 5-card hand, pick one for guess.
   - **Board games (Hold'em, Omaha):** "Hole cards" = 0–2 (Hold'em) or 0–4 (Omaha). We pad with random when needed, simulate random boards. Dead cards excluded.
2. **Get hand:** System runs the simulation, picks one random result from the ranked list.
3. **Guess:** User sees the hand and estimates its percentile (0 = nuts, 100 = junk).
4. **Score:** Green (within 4%), Yellow (4–12%), Red (> 12%). Configurable in `config/training.toml`.
5. **Session stats:** Attempts, accuracy %, current streak.

**Hand generator** — flash cards / study. Flow:
1. Setup: 0–full hole/draw cards (all optional). Dead cards excluded.
2. Generate hand — system runs simulation, picks random result, displays with nice graphics.
3. Shows percentile. "Next" for another. No scoring.

### Simulation Interface (see GAME-RULES.md)

Simulation types: **draw** (0–5 held, complete to 5-card hand) vs **board** (0–held hole + random board). Config in `games.toml`. Add new games by implementing evaluator and setting `simulation_type`.

### Extensibility

The training module defines a trait for training exercises. New exercise types (hand ranking quizzes, range construction drills, equity estimation games) implement this trait and plug into the training session manager. Each exercise type declares its own input requirements, scoring logic, and result format.

### Session Persistence

Training sessions produce result summaries that can be serialized. The persistence layer is abstracted behind a storage trait:

- Local file storage (JSON or SQLite) for the initial implementation.
- Remote/cloud storage as a future option.
- The training module does not know which backend is in use.

---

## Crate: `poker-gpu` (Future)

GPU compute abstraction for expensive equity calculations.

### Purpose

Preflop and postflop equity calculations require evaluating millions to billions of hand matchups. CPU-based Monte Carlo is sufficient for single-hand draw simulations, but full enumeration or range-vs-range equity demands GPU parallelism.

### Approach

- Built on `wgpu` for cross-platform GPU compute (Vulkan, Metal, DX12).
- Lookup tables are uploaded as GPU storage buffers.
- A compute shader implements the state machine traversal in parallel.
- The host (CPU) side handles input preparation and result collection.
- Exposes the same compute trait used by `poker-sim`, so the simulation engine can transparently use GPU when available and fall back to CPU otherwise.

### Integration

- Enabled via a Cargo feature flag (`gpu`). The application compiles and runs without GPU support.
- GPU availability is detected at runtime. If no compatible GPU is found, the engine falls back to CPU automatically.
- The frontend can display a GPU status indicator and allow the user to toggle compute backends.

---

## Crate: `poker-app` (Tauri Shell)

The native application wrapper. Bridges the Rust engine to the frontend via Tauri's IPC command system.

### Responsibilities

- Define Tauri commands that map to engine operations (run simulation, start training session, get game list, etc.).
- Serialize/deserialize command payloads using `serde`. All IPC types are defined in a shared types module.
- Manage application state (current game variant, active training session, user preferences).
- Handle cross-platform build configuration for Windows and macOS.

### Command Design

Each Tauri command is a thin wrapper that validates input, calls into the appropriate engine crate, and returns a typed result. Commands are async where simulation may take noticeable time (large draw counts), allowing the UI to show progress.

Commands are grouped by domain:

- **Simulation commands** — run draw simulation, enumerate best/worst hands.
- **Training commands** — start session, submit guess, get session stats.
- **Config commands** — get available games, get/set user preferences.

### State Management

Application state is managed on the Rust side using Tauri's managed state. The frontend does not maintain authoritative game state — it requests it from the backend. This keeps the source of truth in Rust and simplifies the frontend.

### Preferences Storage (Abstraction)

User preferences (game_id, draw_count, training_mode, training_draw_count) are persisted via a **backend-agnostic** abstraction:

- **`PreferencesStorage` trait** — `load()` and `save(&Preferences)` — hides the persistence format.
- **`StoreBackend`** — current implementation using `tauri-plugin-store` (JSON file in app config dir).
- **`StorageState`** — wraps `Arc<dyn PreferencesStorage>` for dependency injection.

To swap to SQLite or remote: implement `PreferencesStorage` for a new backend, and in `setup()` replace `StoreBackend` with the new implementation when managing `StorageState`. Commands stay unchanged.

---

## Frontend (React + TypeScript)

A Tauri-hosted web frontend providing the user interface. Communicates exclusively through Tauri IPC commands.

### Design Language

- Clean, modern, native-feeling interface. Minimal chrome. Content-first layout.
- Card rendering uses solid colored backgrounds per suit:
  - Clubs: green
  - Diamonds: blue
  - Hearts: red
  - Spades: dark grey/charcoal
- Card faces display bold, high-contrast rank text on the suit-colored background.
- All color values, sizes, and spacing are defined in a theme configuration — no hardcoded colors in components.
- The card component is reusable and consistent across all views (simulation input, results, training).

### View Structure

- **Landing** — Home page with navigation cards to Simulation, Training, Settings.
- **Simulation** — Card input (your draw / hole cards, dead cards), simulation count, Run button. Results table: Rank | Hand | Percentile (sorted). Re-run to get new simulations.
- **Training** — Percentile guessing drill: (1) Setup draw scenario, (2) Get random hand from simulation, (3) Guess percentile. Green/Yellow/Red feedback. Session stats.
- **Settings** — User preferences (placeholder). Future: theme, scoring thresholds, compute backend.
- **Layout** — Sidebar navigation (desktop app style). Active route highlighted.

### Card Input

- Visual card picker. Click + to expand grid. Select cards by clicking—selected cards appear in the held hand row with correct rank (2, 3, … T, J, Q, K, A) and suit color.
- Dead cards use the same picker. Held and dead cards are mutually exclusive in the pool.
- Click a selected card to remove it.

### Results Table

- Sortable columns: Rank, Hand (visual card display), Percentile.
- Percentile values are calculated based on draw count (1,000 draws = 0.1% increments, 100 draws = 1% increments).
- Scrollable for large result sets. Virtualized rendering for performance with 1,000+ rows.

---

## Configuration

All tunable parameters live in configuration files under `config/`. The application reads these at startup and exposes them through typed Rust structs.

### `games.toml`

- Game variant definitions: name, hand size, number of streets, number of draws per street, whether suits matter for evaluation.
- Card constants: ranks available, suits available, deck size.

### `training.toml`

- Scoring thresholds for the percentile guessing game (e.g., exact = 4%, close = 12%).
- Default session length, scoring weights, streak bonuses.

### `ui.toml`

- Suit-to-color mappings.
- Card dimensions, font sizes, spacing.
- Feedback colors (correct, close, incorrect).

---

## Cross-Compilation and Distribution

### Build Targets

- **Windows**: MSVC toolchain. Tauri bundles as `.msi` or `.exe` installer.
- **macOS**: Apple toolchain. Tauri bundles as `.dmg` with universal binary (x86_64 + aarch64).
- **Linux** (future): AppImage or `.deb`.

### CI/CD

- GitHub Actions workflow with matrix builds for Windows and macOS.
- Lookup table generation runs as a pre-build verification step (tables are checked in, CI validates they match the generator output).
- Frontend build (npm) and Rust build (cargo) are orchestrated by Tauri's build system.

### Feature Flags

- `gpu` — enables the `poker-gpu` crate and GPU compute backend. Off by default.
- `training` — enables the training module. On by default.
- Future flags for additional game variants if they carry heavy dependencies.

---

## Data Flow Summary

```
User Input (frontend)
    │
    ▼
Tauri IPC Command
    │
    ▼
poker-app (validates, dispatches)
    │
    ├──► poker-sim (simulation engine)
    │       │
    │       ├──► poker-core (deck, cards, draw)
    │       └──► poker-eval (hand ranking via lookup tables)
    │
    ├──► poker-training (training session management)
    │       │
    │       └──► poker-sim (generates scenarios)
    │
    └──► poker-gpu [optional] (GPU compute for equity)
            │
            └──► poker-eval (lookup tables as GPU buffers)
    │
    ▼
Typed Result (serde)
    │
    ▼
Frontend Renders
```

---

## Open Questions and Future Considerations

- **Lookup table versioning** — if the table format changes, the application needs to detect stale tables and regenerate or prompt for update.
- **Multi-street simulation** — Badugi has 3 draw rounds. The simulation engine will need to model sequential draws with decision points. This may require a strategy/policy trait that the simulator calls at each decision point.
- **Range input** — for equity calculations, users will need to input hand ranges (e.g., "all 2-card draws to a 7-low"). This requires a range definition language or visual range builder.
- **Replay and sharing** — deterministic RNG seeding allows exact simulation replay. Results could be exported and shared.
- **Remote storage** — the storage trait in `poker-training` leaves room for syncing sessions across devices via a remote backend.
