//! zluna — client-agnostic chat backend in front of a local Ollama server.
//!
//! Endpoints
//!   GET  /api/health  -> {"ok":true,"ollama":bool}
//!   GET  /api/models  -> {"models":[..],"default":".."}
//!   POST /api/chat    {"model"?: "..", "messages":[{"role","content"}]}
//!                     -> NDJSON stream: {"delta":".."}* then {"done":true,"tokens":n,"tps":x}
//!                        or {"error":".."}
//!
//! Env: ZLUNA_ADDR (127.0.0.1:3000), OLLAMA_URL (http://127.0.0.1:11434), ZLUNA_MODEL (qwen3:8b)

use std::{convert::Infallible, time::Duration};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

const SYSTEM_PROMPT: &str = "คุณคือ Zluna ผู้ช่วย AI ที่เป็นมิตร ตอบเป็นภาษาไทยเป็นหลัก \
(ยกเว้นผู้ใช้ถามเป็นภาษาอื่น) ตอบกระชับ ชัดเจน และถูกต้อง";

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    ollama: String,
    default_model: String,
}

#[derive(Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    model: Option<String>,
    messages: Vec<Message>,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    let addr = env_or("ZLUNA_ADDR", "127.0.0.1:3000");
    let state = AppState {
        http: reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("build http client"),
        ollama: env_or("OLLAMA_URL", "http://127.0.0.1:11434")
            .trim_end_matches('/')
            .to_string(),
        default_model: env_or("ZLUNA_MODEL", "qwen3:8b"),
    };
    println!("zluna: ollama={} default_model={}", state.ollama, state.default_model);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/models", get(models))
        .route("/api/chat", post(chat))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("bind {addr}: {e}"));
    println!("zluna: listening on http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}

async fn health(State(s): State<AppState>) -> Json<Value> {
    let ollama = s
        .http
        .get(format!("{}/api/version", s.ollama))
        .send()
        .await
        .is_ok_and(|r| r.status().is_success());
    Json(json!({ "ok": true, "ollama": ollama }))
}

async fn models(State(s): State<AppState>) -> Result<Json<Value>, AppError> {
    let tags: Value = s
        .http
        .get(format!("{}/api/tags", s.ollama))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let names: Vec<&str> = tags["models"]
        .as_array()
        .map(|a| a.iter().filter_map(|m| m["name"].as_str()).collect())
        .unwrap_or_default();
    Ok(Json(json!({ "models": names, "default": s.default_model })))
}

async fn chat(
    State(s): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Response, AppError> {
    let model = req
        .model
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| s.default_model.clone());

    // The backend owns the system prompt; clients may only send user/assistant turns.
    let mut messages = vec![Message {
        role: "system".into(),
        content: SYSTEM_PROMPT.into(),
    }];
    messages.extend(
        req.messages
            .into_iter()
            .filter(|m| matches!(m.role.as_str(), "user" | "assistant")),
    );
    if messages.len() == 1 {
        return Err(AppError(StatusCode::BAD_REQUEST, "messages is empty".into()));
    }

    let res = s
        .http
        .post(format!("{}/api/chat", s.ollama))
        .json(&json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "think": false,
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(AppError(StatusCode::BAD_GATEWAY, msg));
    }

    // Re-encode Ollama's NDJSON stream into our own small schema.
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(32);
    tokio::spawn(async move {
        let mut upstream = res.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = upstream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Ok(ndjson(json!({ "error": e.to_string() })))).await;
                    return;
                }
            };
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
                let Some(out) = translate(&v) else { continue };
                // Client went away: dropping `upstream` closes the Ollama request too.
                if tx.send(Ok(ndjson(out))).await.is_err() {
                    return;
                }
            }
        }
    });

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(ReceiverStream::new(rx)))
        .unwrap())
}

/// Map one Ollama stream object to a zluna event (None = nothing to send).
fn translate(v: &Value) -> Option<Value> {
    if let Some(e) = v["error"].as_str() {
        return Some(json!({ "error": e }));
    }
    if v["done"].as_bool() == Some(true) {
        let tokens = v["eval_count"].as_u64().unwrap_or(0);
        let ns = v["eval_duration"].as_u64().unwrap_or(0);
        let tps = if ns > 0 { tokens as f64 / (ns as f64 / 1e9) } else { 0.0 };
        return Some(json!({ "done": true, "tokens": tokens, "tps": (tps * 10.0).round() / 10.0 }));
    }
    match v["message"]["content"].as_str() {
        Some(d) if !d.is_empty() => Some(json!({ "delta": d })),
        _ => None,
    }
}

fn ndjson(v: Value) -> Bytes {
    let mut s = v.to_string();
    s.push('\n');
    Bytes::from(s)
}

struct AppError(StatusCode, String);

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        let msg = if e.is_connect() {
            "cannot reach Ollama — is `ollama serve` running?".to_string()
        } else {
            e.to_string()
        };
        AppError(StatusCode::BAD_GATEWAY, msg)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}
