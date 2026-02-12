#!/usr/bin/env bash
# Replicate the CI release build locally.
# Usage: ./scripts/build-release.sh [version]
#   version: optional, e.g. 1.0.0 (default: from package.json)
#   Set VERSION=1.0.0 in env or pass as first arg to override.

set -e

VERSION="${1:-${VERSION:-}}"
if [[ -n "$VERSION" ]]; then
  echo "Setting version to $VERSION"
  npm pkg set version="$VERSION"
fi

echo "Installing dependencies..."
npm ci

echo "Running tests..."
cargo test --workspace

echo "Installing Tauri CLI..."
npm install @tauri-apps/cli@^2

# Build for current platform (or use TARGET env var)
TARGET="${TARGET:-}"
if [[ -z "$TARGET" ]]; then
  case "$(uname -s)" in
    Darwin)
      case "$(uname -m)" in
        arm64) TARGET=aarch64-apple-darwin ;;
        *)     TARGET=x86_64-apple-darwin ;;
      esac
      ;;
    Linux)   TARGET=x86_64-unknown-linux-gnu ;;
    MINGW*)  TARGET=x86_64-pc-windows-msvc ;;
    *)       echo "Unknown platform"; exit 1 ;;
  esac
fi

echo "Building for $TARGET..."
npx tauri build --target "$TARGET"

echo ""
echo "Build complete. Output:"
case "$(uname -s)" in
  Darwin)
    echo "  DMG: src-tauri/target/$TARGET/release/bundle/dmg/"
    ls -la src-tauri/target/"$TARGET"/release/bundle/dmg/*.dmg 2>/dev/null || true
    ;;
  Linux)
    echo "  deb: src-tauri/target/$TARGET/release/bundle/deb/"
    echo "  AppImage: src-tauri/target/$TARGET/release/bundle/appimage/"
    ;;
  MINGW*)
    echo "  msi/nsis: src-tauri/target/$TARGET/release/bundle/"
    ;;
esac
