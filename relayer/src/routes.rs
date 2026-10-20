//! HTTP routes: health + submit-intent (Days 44–45) + HTTP 402 payer (Day 50).

use axum::extract::{Path, Query, State};
use axum::http::{Method, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pay402::{
    hex_decode_32, hex_encode, parse_pay_body, quote, PayLedger, PayRequest, PayResponse,
    PAID_BODY,
};
use crate::queue::{JobStatus, ProveQueue};

#[derive(Clone)]
pub struct AppState {
    pub queue: ProveQueue,
    pub pay: PayLedger,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Deserialize)]
pub struct SubmitIntentRequest {
    /// Base64 or hex (`0x…`) private intent / witness bytes.
    pub intent_base64: String,
}

#[derive(Serialize)]
pub struct SubmitIntentResponse {
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
        .route("/submit-intent", post(submit_intent))
        .route("/jobs/{id}", get(get_job))
        .route("/402/resource", get(paid_resource))
        .route("/402/pay", post(pay_402))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "zk-spot-shield-relayer",
    })
}

async fn submit_intent(
    State(state): State<AppState>,
    Json(body): Json<SubmitIntentRequest>,
) -> Result<Json<SubmitIntentResponse>, (StatusCode, String)> {
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
    Ok(Json(SubmitIntentResponse {
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

#[derive(Deserialize)]
struct ResourceQuery {
    nullifier: Option<String>,
}

#[derive(Serialize)]
struct PaidResource {
    data: &'static str,
    nullifier_hex: String,
}

/// GET /402/resource — 402 quote unless `?nullifier=` is a paid receipt.
async fn paid_resource(
    State(state): State<AppState>,
    Query(q): Query<ResourceQuery>,
) -> Result<Json<PaidResource>, (StatusCode, Json<crate::pay402::PaymentQuote>)> {
    if let Some(raw) = q.nullifier.as_deref() {
        if let Some(nf) = hex_decode_32(raw) {
            if state.pay.is_paid(&nf).await {
                return Ok(Json(PaidResource {
                    data: PAID_BODY,
                    nullifier_hex: hex_encode(&nf),
                }));
            }
        }
    }
    Err((StatusCode::PAYMENT_REQUIRED, Json(quote())))
}

async fn pay_402(
    State(state): State<AppState>,
    Json(body): Json<PayRequest>,
) -> Result<Json<PayResponse>, (StatusCode, String)> {
    let public = parse_pay_body(&body.journal_base64).map_err(|e| e.status_msg())?;
    state.pay.mark_paid(public.nullifier).await;
    Ok(Json(PayResponse {
        paid: true,
        nullifier_hex: hex_encode(&public.nullifier),
        amount: public.requested_swap_amount,
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
        tokio::spawn(async move {
            crate::queue::run_worker(q, rx, None, crate::fees::FeeConfig::default()).await
        });
        app_router(AppState {
            queue,
            pay: crate::pay402::PayLedger::default(),
        })
    }

    #[tokio::test]
    async fn cors_allows_browser_origin() {
        let app = test_app().await;
        let res = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/health")
                    .header("origin", "http://localhost:5173")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let allow = res.headers().get("access-control-allow-origin");
        assert!(allow.is_some(), "expected CORS header, status={}", res.status());
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
    async fn submit_intent_queues_job() {
        let app = test_app().await;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/submit-intent")
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

    #[tokio::test]
    async fn http_402_then_pay_with_fixture_journal() {
        use base64::Engine;
        let journal = std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../zk-circuit/fixtures/happy/journal.bin"),
        )
        .unwrap();
        let app = test_app().await;
        let unpaid = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/402/resource")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unpaid.status(), StatusCode::PAYMENT_REQUIRED);
        let q: serde_json::Value =
            serde_json::from_slice(&unpaid.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(q["amount"], 100);
        assert!(q["pay_via"].as_str().unwrap().contains("settle_shielded_spot"));

        let b64 = base64::engine::general_purpose::STANDARD.encode(&journal);
        let pay = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/402/pay")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"journal_base64":"{b64}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pay.status(), StatusCode::OK);
        let paid_body: serde_json::Value =
            serde_json::from_slice(&pay.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        let nf = paid_body["nullifier_hex"].as_str().unwrap();
        assert!(!paid_body["nullifier_hex"].as_str().unwrap().contains("wallet"));

        let ok = app
            .oneshot(
                Request::builder()
                    .uri(format!("/402/resource?nullifier={nf}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        let data: serde_json::Value =
            serde_json::from_slice(&ok.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(data["data"], "shielded-402-ok");
    }
}
