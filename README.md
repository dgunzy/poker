# Poker Simulator

A cross-platform desktop app for poker hand simulation and training. Supports NL 2-7 Single Draw, A-5 Lowball / Razz, No Limit Hold'em, Pot Limit Omaha, and Omaha Hi-Lo 8-or-better.

Built with [Tauri 2](https://v2.tauri.app/) (Rust backend) and React (TypeScript frontend).

## Install

Download the latest release for your platform from the [Releases](../../releases) page:

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Poker Simulator_x.x.x_aarch64.dmg` |
| macOS (Intel, e.g. MacBook Pro 2020) | `Poker Simulator_x.x.x_x64.dmg` |
| Windows | `Poker Simulator_x.x.x_x64-setup.exe` or `.msi` |
| Linux (Debian/Ubuntu) | `poker-simulator_x.x.x_amd64.deb` |
| Linux (other) | `poker-simulator_x.x.x_amd64.AppImage` |

**macOS**: If you see "app is damaged" or "app can't be opened because it is from an unidentified developer", right-click the app and choose **Open**, then click Open in the dialog. (The app uses ad-hoc signing; double-click may be blocked until you whitelist it.)

**Windows**: If SmartScreen warns about an unrecognized app, click "More info" then "Run anyway". This happens because the app is not code-signed.

**Linux**: For AppImage, make it executable first: `chmod +x *.AppImage && ./*.AppImage`

## Supported Games

- **NL 2-7 Single Draw** — Deuce-to-Seven lowball. Aces high, straights/flushes count against you. Best hand: 7-5-4-3-2.
- **A-5 / Razz** — Ace-to-Five lowball. Aces low, straights/flushes ignored. Best hand: A-2-3-4-5 (wheel).
- **NLHE (No Limit Hold'em)** — 2 hole cards + 5 community cards. Standard high hand rankings.
- **PLO (Pot Limit Omaha)** — 4 hole cards + 5 board. Must use exactly 2 from hand, 3 from board.
- **O8 (Omaha Hi-Lo 8-or-better)** — Split pot. High + low (5 unpaired cards 8-or-below to qualify).

## Features

- **Simulation**: Run Monte Carlo simulations (1 to 100,000 draws) with held cards, dead cards, and configurable game variants. Results show hand rankings and percentiles.
- **Training — Percentile Guessing**: A random hand is dealt and you guess where it ranks (0% = best, 100% = worst). Feedback: green (within 4%), yellow (4-12%), red (>12%).
- **Training — Hand Generator**: Flash-card style hand generation for study. Shows hand + percentile.
- **Board game training**: For Hold'em/PLO/O8, training deals hole cards + a board and rates your hand against random opponents on that same board.
- **Split pot training**: O8 shows both high and low percentiles independently.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- Platform-specific system dependencies for Tauri — see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Microsoft Visual Studio C++ Build Tools, WebView2 (pre-installed on Windows 10+)
  - **Linux (Debian/Ubuntu)**:
    ```bash
    sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf
    ```

### Setup

```bash
npm install        # Install frontend dependencies
npm run tauri dev  # Compile Rust + launch app with hot reload
```

Or with Make:

```bash
make install   # npm install
make run       # npm run tauri dev
```

### Build

```bash
npm run tauri build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`.

### Test

```bash
cargo test --workspace   # Run all Rust tests (core, eval, sim, training)
```

## Project Structure

```
poker/
├── crates/
│   ├── poker-core/      # Card, Deck, Hand types
│   ├── poker-eval/      # Hand evaluators (2-7, A-5, Hold'em, Omaha, O8)
│   └── poker-sim/       # Monte Carlo simulation engine + training strategies
├── src-tauri/           # Tauri app shell (Rust IPC commands, storage)
├── src/                 # React frontend (TypeScript)
│   ├── api.ts           # Tauri IPC abstraction layer
│   ├── pages/           # Simulation, Training, Settings
│   ├── components/      # CardPicker, CardDisplay, Layout
│   └── context/         # Game selection context
├── config/              # TOML config (games, training thresholds, UI colors)
└── .github/workflows/   # CI + release builds
```

### Architecture

The Rust backend is split into three crates:

- **poker-core**: Compact card encoding (0-51, bit-packed rank/suit), deck with Fisher-Yates shuffle, seeded RNG for reproducibility.
- **poker-eval**: Pluggable evaluators behind an `Evaluator` trait. Each game variant produces a numeric rank (lower = stronger). Supports standard high, lowball, and split-pot evaluation.
- **poker-sim**: `Simulator` for Monte Carlo draws and `TrainingStrategy` trait for game-specific training scenario generation. Uses Rayon for parallel execution.

The frontend communicates with Rust via Tauri IPC commands. All card data is serialized as `{rank: string, suit: string}` over the bridge.

## Releasing

Releases are built automatically by GitHub Actions when a version tag is pushed:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers builds for macOS (ARM + Intel), Windows, and Linux. Artifacts are uploaded as a draft GitHub Release for review before publishing. The version is read from the tag (e.g. `v1.0.0` → `1.0.0`).

### Test the release build locally

Before pushing a tag, you can replicate the CI build locally:

```bash
make build-release VERSION=1.0.0
# or: ./scripts/build-release.sh 1.0.0
```

The DMG (on macOS) will be in `src-tauri/target/<target>/release/bundle/dmg/`. To test: open the DMG, drag the app to Applications, then **right-click → Open** (ad-hoc signing allows opening via right-click; double-click may still show a Gatekeeper prompt until you whitelist it).

## License

MIT
