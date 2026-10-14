//! HTTP routes: health + submit-proof (Days 44–45).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::queue::{JobStatus, ProveQueue};

#[derive(Clone)]
pub struct AppState {
    pub queue: ProveQueue,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Deserialize)]
pub struct SubmitProofRequest {
    /// Base64 or raw hex private intent; for Day 45 we accept raw bytes as base64.
    pub intent_base64: String,
}

#[derive(Serialize)]
pub struct SubmitProofResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub intent_len: usize,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub intent_len: usize,
    pub error: Option<String>,
}

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/submit-proof", post(submit_proof))
        .route("/jobs/{id}", get(get_job))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "zk-spot-shield-relayer",
    })
}

async fn submit_proof(
    State(state): State<AppState>,
    Json(body): Json<SubmitProofRequest>,
) -> Result<Json<SubmitProofResponse>, (StatusCode, String)> {
    use base64_decode::decode_intent;

    let intent = decode_intent(&body.intent_base64)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    if intent.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "intent must be non-empty".into()));
    }
    if intent.len() > 256 * 1024 {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "intent too large".into()));
    }

    let job = state.queue.enqueue(intent).await;
    Ok(Json(SubmitProofResponse {
        job_id: job.id,
        status: job.status,
        intent_len: job.intent_len,
    }))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobResponse>, StatusCode> {
    let job = state.queue.get(id).await.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(JobResponse {
        job_id: job.id,
        status: job.status,
        intent_len: job.intent_len,
        error: job.error,
    }))
}

/// Minimal base64 (std alphabet) without pulling another crate into the hot path.
mod base64_decode {
    pub fn decode_intent(s: &str) -> Result<Vec<u8>, String> {
        // Prefer hex `0x…` for small fixtures; else standard base64.
        let t = s.trim();
        if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            if !hex.len().is_multiple_of(2) {
                return Err("hex length must be even".into());
            }
            let mut out = Vec::with_capacity(hex.len() / 2);
            for i in (0..hex.len()).step_by(2) {
                let b = u8::from_str_radix(&hex[i..i + 2], 16)
                    .map_err(|_| "invalid hex".to_string())?;
                out.push(b);
            }
            return Ok(out);
        }
        decode_base64(t)
    }

    fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
        // Use engine from `base64` if available — keep zero deps: simple table.
        fn val(c: u8) -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                b'=' => Some(0),
                _ => None,
            }
        }
        let bytes = input.as_bytes();
        if !bytes.len().is_multiple_of(4) {
            return Err("invalid base64 length".into());
        }
        let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
        for chunk in bytes.chunks(4) {
            let (a, b, c, d) = (
                val(chunk[0]).ok_or("invalid base64")?,
                val(chunk[1]).ok_or("invalid base64")?,
                val(chunk[2]).ok_or("invalid base64")?,
                val(chunk[3]).ok_or("invalid base64")?,
            );
            out.push((a << 2) | (b >> 4));
            if chunk[2] != b'=' {
                out.push((b << 4) | (c >> 2));
            }
            if chunk[3] != b'=' {
                out.push((c << 6) | d);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let (queue, rx) = ProveQueue::new();
        let q = queue.clone();
        tokio::spawn(async move { crate::queue::run_worker(q, rx).await });
        app_router(AppState { queue })
    }

    #[tokio::test]
    async fn health_ok() {
        let app = test_app().await;
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn submit_proof_queues_job() {
        let app = test_app().await;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/submit-proof")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"intent_base64":"0x010203"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["intent_len"], 3);
        assert!(v["job_id"].as_str().is_some());
    }
}
