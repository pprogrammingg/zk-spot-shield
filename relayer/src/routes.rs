//! HTTP routes: health + submit-intent + HTTP 402 payer.

use axum::extract::{Path, Query, State};
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::pay402::{
    accept_public_pay, delivered_banner, delivered_public_banner, describe_pay_attempt,
    hex_decode_32, hex_encode, pay_banner, pay_public_banner, quote_banner, quote_for,
    receipt_lookup_banner, PayLedger, PayPublicRequest, PayPublicResponse, PayRequest, PayResponse,
    PublicPayLedger, QuoteSlot, PAID_BODY, PAID_PUBLIC_BODY,
};
use crate::queue::{JobStatus, ProveQueue};

#[derive(Clone)]
pub struct AppState {
    pub queue: ProveQueue,
    pub pay: PayLedger,
    pub pay_public: PublicPayLedger,
    pub quote: QuoteSlot,
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
        .route("/402/pay-public", post(pay_402_public))
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

    let intent = decode_intent(&body.intent_base64).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
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
    signature: Option<String>,
    amount: Option<u64>,
    mint: Option<String>,
}

#[derive(Serialize)]
struct PaidResource {
    data: &'static str,
    nullifier_hex: String,
}

#[derive(Serialize)]
struct PaidPublicResource {
    data: &'static str,
    signature: String,
    unlock: &'static str,
}

/// GET /402/resource — 402 quote unless `?nullifier=` or `?signature=` is paid.
/// Optional `?amount=` and `?mint=0x…` set the outstanding quote (vault mint A on Devnet).
async fn paid_resource(
    State(state): State<AppState>,
    Query(q): Query<ResourceQuery>,
) -> axum::response::Response {
    if let Some(raw) = q.signature.as_deref() {
        let sig = raw.trim();
        if !sig.is_empty() && state.pay_public.is_paid(sig).await {
            tracing::info!("{}", delivered_public_banner(sig));
            return Json(PaidPublicResource {
                data: PAID_PUBLIC_BODY,
                signature: sig.to_string(),
                unlock: "wallet",
            })
            .into_response();
        }
        if !sig.is_empty() {
            tracing::info!("402 public receipt lookup — not paid yet ({sig})");
        }
    }
    if let Some(raw) = q.nullifier.as_deref() {
        if let Some(nf) = hex_decode_32(raw) {
            if state.pay.is_paid(&nf).await {
                tracing::info!("{}", delivered_banner(&nf));
                return Json(PaidResource {
                    data: PAID_BODY,
                    nullifier_hex: hex_encode(&nf),
                })
                .into_response();
            }
            tracing::info!("{}", receipt_lookup_banner(&nf, false));
        } else {
            tracing::info!("402 receipt lookup — not a 32-byte nullifier ({raw})");
        }
    }

    let (mut amount, mut mint) = state.quote.get().await;
    if let Some(a) = q.amount {
        if a == 0 {
            return (StatusCode::BAD_REQUEST, "amount must be > 0").into_response();
        }
        amount = a;
    }
    if let Some(raw) = q.mint.as_deref() {
        let Some(m) = hex_decode_32(raw) else {
            return (StatusCode::BAD_REQUEST, "mint must be 32-byte 0x-hex").into_response();
        };
        mint = m;
    }
    if q.amount.is_some() || q.mint.is_some() {
        state.quote.set(amount, mint).await;
    }
    tracing::info!("{}", quote_banner(amount, &mint));
    (
        StatusCode::PAYMENT_REQUIRED,
        [("payment-required", "zk-spot-shield; unlock=nullifier")],
        Json(quote_for(amount, mint)),
    )
        .into_response()
}

async fn pay_402(
    State(state): State<AppState>,
    Json(body): Json<PayRequest>,
) -> Result<Json<PayResponse>, (StatusCode, String)> {
    let (amount, mint) = state.quote.get().await;
    let (inspect, parsed) = describe_pay_attempt(&body.journal_base64, amount, mint);
    let public = match parsed {
        Ok(p) => {
            tracing::info!("{inspect}");
            p
        }
        Err(e) => {
            tracing::warn!("{inspect}");
            tracing::warn!("402 pay rejected: {}", e.status_msg().1);
            return Err(e.status_msg());
        }
    };
    state.pay.mark_paid(public.nullifier).await;
    tracing::info!("{}", pay_banner(public.requested_swap_amount, &public.nullifier));
    Ok(Json(PayResponse {
        paid: true,
        nullifier_hex: hex_encode(&public.nullifier),
        amount: public.requested_swap_amount,
    }))
}

async fn pay_402_public(
    State(state): State<AppState>,
    Json(body): Json<PayPublicRequest>,
) -> Result<Json<PayPublicResponse>, (StatusCode, String)> {
    let (amount, _mint) = state.quote.get().await;
    if let Err(e) = accept_public_pay(&body, amount) {
        tracing::warn!("402 pay-public rejected: {}", e.status_msg().1);
        return Err(e.status_msg());
    }
    let signature = body.signature.trim().to_string();
    let payer = body.payer.trim().to_string();
    state.pay_public.mark_paid(signature.clone()).await;
    tracing::info!("{}", pay_public_banner(body.amount, &signature, &payer));
    Ok(Json(PayPublicResponse {
        paid: true,
        signature,
        payer,
        amount: body.amount,
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
            pay_public: crate::pay402::PublicPayLedger::default(),
            quote: crate::pay402::QuoteSlot::default(),
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
        assert!(
            allow.is_some(),
            "expected CORS header, status={}",
            res.status()
        );
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
        assert_eq!(
            unpaid.headers().get("payment-required").unwrap(),
            "zk-spot-shield; unlock=nullifier"
        );
        let q: serde_json::Value =
            serde_json::from_slice(&unpaid.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(q["amount"], 100);
        assert_eq!(q["http_status"], 402);
        assert!(q["unlock"].as_str().unwrap().contains("nullifier"));
        assert!(q["contrast"].as_str().unwrap().contains("x402"));
        assert!(q["contrast"]
            .as_str()
            .unwrap()
            .contains("without a wallet on the receipt"));
        assert!(q["pay_via"]
            .as_str()
            .unwrap()
            .contains("settle_shielded_spot"));

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
            serde_json::from_slice(&pay.into_body().collect().await.unwrap().to_bytes()).unwrap();
        let nf = paid_body["nullifier_hex"].as_str().unwrap();
        assert!(!paid_body["nullifier_hex"]
            .as_str()
            .unwrap()
            .contains("wallet"));

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
            serde_json::from_slice(&ok.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(data["data"], "shielded-402-ok");
    }

    #[tokio::test]
    async fn http_402_custom_amount_mint_then_pay() {
        use base64::Engine;
        let mut journal = std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../zk-circuit/fixtures/happy/journal.bin"),
        )
        .unwrap();
        journal[0..8].copy_from_slice(&100_000u64.to_le_bytes());
        let mint = [0xab; 32];
        journal[8..40].copy_from_slice(&mint);
        let mint_hex = crate::pay402::hex_encode(&mint);
        let app = test_app().await;

        let unpaid = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/402/resource?amount=100000&mint={mint_hex}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unpaid.status(), StatusCode::PAYMENT_REQUIRED);
        let q: serde_json::Value =
            serde_json::from_slice(&unpaid.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(q["amount"], 100000);
        assert_eq!(q["mint_hex"], mint_hex);

        let b64 = base64::engine::general_purpose::STANDARD.encode(&journal);
        let pay = app
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
    }

    #[tokio::test]
    async fn http_402_pay_public_then_receipt() {
        let app = test_app().await;
        let unpaid = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/402/resource?amount=66")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unpaid.status(), StatusCode::PAYMENT_REQUIRED);

        let pay = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/402/pay-public")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"signature":"PubSig111","payer":"PubPayer111","amount":66}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pay.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&pay.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(body["paid"], true);
        assert_eq!(body["signature"], "PubSig111");

        let ok = app
            .oneshot(
                Request::builder()
                    .uri("/402/resource?signature=PubSig111")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        let data: serde_json::Value =
            serde_json::from_slice(&ok.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(data["data"], "public-402-ok");
        assert_eq!(data["unlock"], "wallet");
    }
}
