# Zluna API

## Contract

```text
Transport
==========

HTTP on ZLUNA_ADDR (default 127.0.0.1:3000). JSON request bodies.
CORS is permissive, so any client can call it, including browser pages opened as file://.
Errors are JSON {"error": "..."} with status 400 (bad request) or
502 (Ollama unreachable or rejected the request).

GET /api/health
----------

Response: {"ok": true, "ollama": bool}
ollama is true when Ollama answers /api/version.

GET /api/models
----------

Response: {"models": ["qwen3:8b", ...], "default": "qwen3:8b"}
models lists the models installed in Ollama. default is ZLUNA_MODEL; it may be
missing from models if it has not been pulled.

POST /api/chat
----------

Request: {"model": "qwen3:8b", "messages": [{"role": "user", "content": "..."}]}
model is optional; empty or missing uses ZLUNA_MODEL.
messages is the full conversation; the client keeps history.
Only user and assistant roles are accepted; other roles are dropped.
The backend prepends its own system prompt. Empty messages returns 400.

Response: application/x-ndjson, one JSON object per line:
  {"delta": "..."}                          text chunk, append to the reply
  {"done": true, "tokens": n, "tps": x}     end of reply with generation stats
  {"error": "..."}                          stream failed; stop reading

Closing the connection cancels generation in Ollama.
```

## Examples

```bash
# ---------- bash ---------- #

# Health and models
curl -s localhost:3000/api/health
curl -s localhost:3000/api/models

# Streaming chat
curl -sN localhost:3000/api/chat \
  -H 'content-type: application/json' \
  -d '{"messages":[{"role":"user","content":"สวัสดี แนะนำตัวหน่อย"}]}'
```
