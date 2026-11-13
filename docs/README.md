# Zluna

## Overview

```text
Local Thai chat
==========

A local LLM chat service that speaks Thai. Everything runs on this machine;
no cloud API or API key is used.

zluna (this repo) is the backend: a small Rust HTTP server that owns the
system prompt and model defaults, and streams replies over HTTP.
It is client-agnostic: any HTTP client can use the API in api.md.
Ollama runs the model on the local GPU.

Flow
----------

client  --HTTP/NDJSON-->  zluna :3000  --HTTP-->  Ollama :11434  -->  GPU

Current state
----------

Backend and Ollama user-local install are implemented and verified end to end.
Default model is qwen3:8b (Q4_K_M, 5.2 GB): 37/37 layers on GPU, about 5.9 GB VRAM
with the 4096-token context, about 57 tokens/s after warm-up. The first request
after Ollama starts is slower (model load and CUDA warm-up).
```

## Documentation

| Document | Purpose |
|---|---|
| [Setup and run](../README.md) | Main entry point for install and run commands |
| [Runtime](runtime.md) | Ollama install, models, configuration, removal, and verification |
| [API contract](api.md) | HTTP endpoints and the streaming event format |
| [Docs format guide](docs-format-guide.md) | Layout examples for project documentation |

## Notes

```text
Documentation policy
==========

Write documentation in English. Keep it concise and immediately understandable.
Put essential information first. Place optional explanations and extended detail
near the end of each file for readers who want the full context.

Follow docs-format-guide.md.

API changes
----------

Whenever an endpoint or stream event changes, update api.md in the same change.

Commits
----------

Commit format: 1 line en https://www.conventionalcommits.org/en/v1.0.0
```

## Details

```text
Why these tools
==========

Machine: RTX 5050 (8 GB VRAM), 30 GB RAM, 12 CPU threads, no sudo.

Ollama gives CUDA acceleration, model management, and a streaming HTTP API with
no build step. It is installed as a user-local tarball, so it needs no sudo and
is removed by deleting two directories.

Docker was considered for easy removal, but GPU access in containers needs
nvidia-container-toolkit (sudo). Without it the model runs on CPU only and is
several times slower, so Docker is not used.

Model choice
----------

qwen3:8b has strong Thai support and fits fully in VRAM at Q4.
zluna sends think=false so Qwen3 answers directly without a reasoning block.
For a Thai-specialised alternative, pull a Typhoon model (see runtime.md).
```
