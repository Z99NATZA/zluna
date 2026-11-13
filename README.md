# zluna

## Setup

```bash
# ---------- bash ---------- #

# Install Ollama user-locally (no sudo) and pull the model.
# Full steps and paths: docs/runtime.md
mkdir -p ~/.local/ollama
curl -fL -o ~/.local/ollama/ollama.tar.zst https://ollama.com/download/ollama-linux-amd64.tar.zst
tar --zstd -xf ~/.local/ollama/ollama.tar.zst -C ~/.local/ollama && rm ~/.local/ollama/ollama.tar.zst
~/.local/ollama/bin/ollama serve & ~/.local/ollama/bin/ollama pull qwen3:8b
```

## Run

```bash
# ---------- bash ---------- #

# Backend on http://127.0.0.1:3000 (starts Ollama if needed)
./run.sh
```

[Runtime and configuration](docs/runtime.md)

[Documentation](docs/README.md)
