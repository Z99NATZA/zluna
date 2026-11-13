#!/usr/bin/env bash
# Resumable model download for slow or unstable connections.
#
# `ollama pull` splits a model into many parallel parts; on a slow link they
# stall and retry, wasting bandwidth. This script downloads the large model
# layer with a single resumable curl stream, verifies its sha256, moves it into
# Ollama's blob store, then runs `ollama pull` to fetch the small remaining files.
#
# Stop it any time (Ctrl+C, reboot); run the same command again to continue.
#
#   scripts/pull-model.sh qwen3:8b
#   scripts/pull-model.sh scb10x/typhoon2.1-gemma3-4b
set -euo pipefail

MODEL="${1:?usage: $0 <model[:tag]>}"
OLLAMA_BIN="${OLLAMA_BIN:-$HOME/.local/ollama/bin/ollama}"
MODELS_DIR="${OLLAMA_MODELS:-$HOME/.ollama/models}"
# Kept outside blobs/ so Ollama's startup prune never deletes a partial file.
DL_DIR="$HOME/.ollama/downloads"
REGISTRY="https://registry.ollama.ai/v2"

name="${MODEL%%:*}"
tag="latest"
[[ "$MODEL" == *:* ]] && tag="${MODEL#*:}"
[[ "$name" == */* ]] || name="library/$name"

echo "▶ manifest $name:$tag"
manifest=$(curl -fsSL -H 'Accept: application/vnd.docker.distribution.manifest.v2+json' \
  "$REGISTRY/$name/manifests/$tag")

# Largest layer = the model weights.
read -r digest size < <(python3 -I -c '
import json, sys
layers = json.loads(sys.argv[1])["layers"]
big = max(layers, key=lambda l: l["size"])
print(big["digest"], big["size"])' "$manifest")

blob="$MODELS_DIR/blobs/${digest/:/-}"
part="$DL_DIR/${digest/:/-}.part"
mkdir -p "$DL_DIR" "$MODELS_DIR/blobs"

if [[ -f "$blob" ]]; then
  echo "▶ weights already present: $blob"
else
  have=$(stat -c %s "$part" 2>/dev/null || echo 0)
  echo "▶ weights $digest"
  echo "  $(( have / 1024 / 1024 )) / $(( size / 1024 / 1024 )) MB done, resuming"
  until curl -fL --retry 20 --retry-delay 5 --retry-all-errors -C - \
      -o "$part" "$REGISTRY/$name/blobs/$digest"; do
    echo "  curl failed; retrying in 10s (Ctrl+C to stop, rerun later to resume)"
    sleep 10
  done

  echo "▶ verifying sha256"
  actual="sha256:$(sha256sum "$part" | cut -d' ' -f1)"
  if [[ "$actual" != "$digest" ]]; then
    echo "  checksum mismatch; deleting $part — rerun to download again" >&2
    rm -f "$part"
    exit 1
  fi
  mv "$part" "$blob"
fi

if ! curl -s -o /dev/null http://127.0.0.1:11434/api/version; then
  echo "▶ starting ollama serve (log: /tmp/ollama.log)"
  nohup "$OLLAMA_BIN" serve >/tmp/ollama.log 2>&1 &
  until curl -s -o /dev/null http://127.0.0.1:11434/api/version; do sleep 0.5; done
fi

echo "▶ ollama pull $MODEL (small remaining files)"
"$OLLAMA_BIN" pull "$MODEL"
