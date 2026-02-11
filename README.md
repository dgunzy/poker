# Poker Simulator

A cross-platform poker simulation and training application for NL 2-7 Single Draw (Deuce-to-Seven Lowball).

## Structure

```
poker/
├── crates/
│   ├── poker-core/    # Card, Deck, Hand types
│   ├── poker-eval/    # 2-7 lowball hand evaluator
│   └── poker-sim/     # Monte Carlo simulation engine
├── src-tauri/         # Tauri app shell
├── src/               # React frontend
└── config/            # Game, training, UI config
```

## Prerequisites

- Rust (rustup)
- Node.js 18+
- Tauri system deps (see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/))

## Run

```bash
make install   # First time only
make run      # Compile and run the desktop app
```

Or without Make:

```bash
npm install
npm run tauri dev
```

## Build

```bash
make build
```

## Test

```bash
cargo test
```
