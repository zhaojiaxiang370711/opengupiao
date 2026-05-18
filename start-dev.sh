#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

# Python environment
export VIRTUAL_ENV="$ROOT/.venv"
export PATH="$VIRTUAL_ENV/bin:$PATH"
export LD_LIBRARY_PATH="$HOME/.local/share/uv/python/cpython-3.13.12-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"
export PYTHONPATH="$VIRTUAL_ENV/lib/python3.13/site-packages:$ROOT/OpenBB/openbb_platform:$PYTHONPATH"

# Database
export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/aaagupiao}"
export REDIS_URL="${REDIS_URL:-redis://localhost:6379}"
export BIND_ADDR="${BIND_ADDR:-0.0.0.0:8080}"
export RUST_LOG="${RUST_LOG:-info}"

echo "=== Starting Rust Backend ==="
cd "$ROOT/backend"
cargo run --release &
BACKEND_PID=$!

echo "=== Starting Vue Frontend ==="
cd "$ROOT/frontend"
npm run dev &
FRONTEND_PID=$!

trap "kill $BACKEND_PID $FRONTEND_PID 2>/dev/null" EXIT
wait
