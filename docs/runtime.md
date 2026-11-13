# Runtime

## Setup

```text
Requirements
==========

NVIDIA GPU with a working driver (nvidia-smi), Rust/Cargo, curl, zstd.
No sudo is needed. Ollama is installed under the user's home directory.

Paths
----------

Ollama binary and CUDA libs:   ~/.local/ollama  (bin/ollama, lib/ollama)
Downloaded models:             ~/.ollama/models
Resumable downloads:           ~/.ollama/downloads  (scripts/pull-model.sh)
Ollama log (from run.sh):      /tmp/ollama.log
```

```bash
# ---------- bash ---------- #

# Install Ollama user-locally
mkdir -p ~/.local/ollama
curl -fL -o ~/.local/ollama/ollama.tar.zst https://ollama.com/download/ollama-linux-amd64.tar.zst
tar --zstd -xf ~/.local/ollama/ollama.tar.zst -C ~/.local/ollama
rm ~/.local/ollama/ollama.tar.zst

# Optional: put ollama on PATH
ln -sf ~/.local/ollama/bin/ollama ~/.local/bin/ollama

# Pull the default model (about 5.2 GB)
~/.local/ollama/bin/ollama serve &
~/.local/ollama/bin/ollama pull qwen3:8b
```

## Run

```bash
# ---------- bash ---------- #

# Start Ollama (if not running) and the zluna backend on :3000
./run.sh
```

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| ZLUNA_ADDR | 127.0.0.1:3000 | Backend listen address |
| OLLAMA_URL | http://127.0.0.1:11434 | Ollama server |
| ZLUNA_MODEL | qwen3:8b | Model used when the client sends none |
| OLLAMA_BIN | ~/.local/ollama/bin/ollama | Ollama binary used by run.sh |

The system prompt is SYSTEM_PROMPT in src/main.rs.

## Models

| Model | Size | Notes |
|---|---|---|
| qwen3:8b | ~5.2 GB | Default. Good Thai, fits fully in 8 GB VRAM |
| qwen3:4b | ~2.6 GB | Faster, lighter, weaker answers |
| scb10x/typhoon2.1-gemma3-4b | ~2.5 GB | Thai-specialised (SCB 10X Typhoon) |

```bash
# ---------- bash ---------- #

# Add or remove a model; GET /api/models lists installed models
~/.local/ollama/bin/ollama pull scb10x/typhoon2.1-gemma3-4b
~/.local/ollama/bin/ollama rm qwen3:4b
~/.local/ollama/bin/ollama list

# Slow or unstable connection: single-stream resumable download.
# Stop any time; run the same command again to continue where it left off.
# Partial data: ~/.ollama/downloads/<digest>.part
scripts/pull-model.sh qwen3:8b
```

## Removal

```bash
# ---------- bash ---------- #

# Stop everything, then delete Ollama and all downloaded models
pkill -f 'ollama serve'
rm -rf ~/.local/ollama ~/.ollama ~/.local/bin/ollama
```

## Verification

```bash
# ---------- bash ---------- #

cargo build --release

# GPU in use: expect "offloaded 37/37 layers to GPU" and PROCESSOR "100% GPU"
grep -i offloaded /tmp/ollama.log
ollama ps

# Speed: the last stream event reports tps (about 57 for qwen3:8b after warm-up)

# API checks: see api.md for examples
curl -s localhost:3000/api/health
```
