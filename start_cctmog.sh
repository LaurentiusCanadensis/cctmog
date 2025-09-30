#!/usr/bin/env zsh
set -euo pipefail

# === CONFIG ===
# Path to your repo root (change if needed or pass as $1)
REPO="${1:-$HOME/RustroverProjects/cctmog}"
PORT="${PORT:-9001}"     # websocket port your server uses

# Binaries (expected by the project you're running)
COMBINED_BIN="cctmog-combined"

# === CLEANUP OLD PROCESSES ===
echo "Killing old processes and freeing port :$PORT (if any)..."

# Kill by process name (ignore errors if none)
pkill -f "cctmog-server" 2>/dev/null || true
pkill -f "cctmog" 2>/dev/null || true
pkill -f "$COMBINED_BIN" 2>/dev/null || true

# Kill anything bound to the WS port
if lsof -ti tcp:"$PORT" >/dev/null 2>&1; then
  lsof -ti tcp:"$PORT" | xargs kill -9 || true
fi

# A tiny pause to let processes die
sleep 0.5

# === BUILD ===
echo "Building combined binary…"
pushd "$REPO" >/dev/null
# Build the combined binary
cargo build --bin "$COMBINED_BIN"
popd >/dev/null

# === LAUNCH USING COMBINED BINARY ===
echo "Starting server and 2 clients using combined binary…"

pushd "$REPO" >/dev/null
# Use the combined binary to start both server and clients
cargo run --bin "$COMBINED_BIN" -- both --port "$PORT" --clients 2
popd >/dev/null

echo "All set. Server + 2 clients started using combined binary."
echo "Tip: You can also run them separately:"
echo "  cargo run --bin $COMBINED_BIN -- server --port $PORT"
echo "  cargo run --bin $COMBINED_BIN -- client"