#!/usr/bin/env bash
# Start Ollama (if not already running) and the zluna backend.
set -e
OLLAMA_BIN="${OLLAMA_BIN:-$HOME/.local/ollama/bin/ollama}"
cd "$(dirname "$0")"

if ! curl -s -o /dev/null http://127.0.0.1:11434/api/version; then
  echo "▶ starting ollama serve (log: /tmp/ollama.log)"
  nohup "$OLLAMA_BIN" serve >/tmp/ollama.log 2>&1 &
  OLLAMA_PID=$!
  trap 'kill $OLLAMA_PID 2>/dev/null' EXIT
  until curl -s -o /dev/null http://127.0.0.1:11434/api/version; do sleep 0.5; done
fi

cargo run --release
