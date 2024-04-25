use agent_issuance::offer::event::OfferEvent;
use agent_verification::connection::event::ConnectionEvent;
use axum::{routing::post, Json, Router};
use reqwest::Client;

const OFFER_ID: &str = "my-first-offer";
const UNICORE: &str = "http://192.168.10.175:3033";

async fn oid4vci() {
    let client = Client::new();

    // Returns a Credential Offer as a form url encoded string to be rendered as a QR Code somewhere.
    let response = client
        .post(format!("{UNICORE}/v1/offers"))
        .json(&serde_json::json!({
            "offerId": OFFER_ID
        }))
        .send()
        .await
        .unwrap();

    let body = response.bytes().await.unwrap();
    let form_url_encoded_credential_offer: String = String::from_utf8(body.to_vec()).unwrap();

    println!(
        "form_url_encoded_credential_offer: {}",
        form_url_encoded_credential_offer
    );
}

async fn siopv2() {
    let client = Client::new();

    // Returns a SIOPv2 Authorization Request as a form url encoded string to be rendered as a QR Code somewhere.
    let response = client
        .post(format!("{UNICORE}/v1/authorization_requests"))
        // `OFFER_ID` can be used as a nonce here, but for proper use, `nonce` should be used only once.
        .json(&serde_json::json!({
        "nonce": OFFER_ID
        }))
        .send()
        .await
        .unwrap();

    let body = response.bytes().await.unwrap();
    let form_url_encoded_credential_offer: String = String::from_utf8(body.to_vec()).unwrap();

    println!(
        "form_url_encoded_authorization_request: {}",
        form_url_encoded_credential_offer
    );
}

async fn oid4vp() {
    let client = Client::new();

    // Returns a OID4VP Authorization Request as a form url encoded string to be rendered as a QR Code somewhere.
    let response = client
        .post(format!("{UNICORE}/v1/authorization_requests"))
        .json(&serde_json::json!({
            "nonce": OFFER_ID,
            "presentation_definition_id": "selv_presentation_definition"
        }))
        .send()
        .await
        .unwrap();

    let body = response.bytes().await.unwrap();
    let form_url_encoded_credential_offer: String = String::from_utf8(body.to_vec()).unwrap();

    println!(
        "form_url_encoded_authorization_request_with_presentation_definition: {}",
        form_url_encoded_credential_offer
    );
}

#[tokio::main]
async fn main() {
    // Prepare some form url encoded strings to be rendered as QR Codes.
    oid4vci().await;
    siopv2().await;
    oid4vp().await;

    // build our application with a route
    let app = Router::new().route("/event-listener", post(event_listener));

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7777").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn event_listener(Json(payload): Json<serde_json::Value>) -> String {
    println!("Received: {:?}", payload);

    // Three types of events to listen to:
    // 1. Credential Request Verified
    // 2. SIOPv2 Authorization Response Verified --> `id_token`
    // 3. OID4VP Authorization Response Verified --> `vp_token` (Verifiable Presentation)

    if let Ok(OfferEvent::CredentialRequestVerified {
        offer_id,
        // Optionally use this `subject_id` to include it in the credential you're about to send back to `UniCore`.
        subject_id: _subject_did,
    }) = serde_json::from_value::<OfferEvent>(payload.clone())
    {
        assert_eq!(offer_id, OFFER_ID);

        let client = Client::new();

        // Post an already signed credential to the `/v1/credentials` endpoint. I just used a hardcoded credential here.
        client
            .post(format!("{UNICORE}/v1/credentials"))
            .json(&serde_json::json!({
                "offerId": offer_id,
                "credential": "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSIsImtpZCI6ImRpZDppb3RhOnJtczoweDQyYWQ1ODgzMjJlNThiM2MwN2FhMzllNDk0OGQwMjFlZTE3ZWNiNTc0NzkxNWU5ZTFmMzVmMDI4ZDdlY2FmOTAjYlFLUVJ6YW9wN0NnRXZxVnE4VWxnTEdzZEYtUi1obkxGa0tGWnFXMlZOMCJ9.eyJpc3MiOiJkaWQ6aW90YTpybXM6MHg0MmFkNTg4MzIyZTU4YjNjMDdhYTM5ZTQ5NDhkMDIxZWUxN2VjYjU3NDc5MTVlOWUxZjM1ZjAyOGQ3ZWNhZjkwIiwic3ViIjoiZGlkOmtleTp6Nk1raDJ5ZVRlUmp2Z2N2TmU0eWVRdEc4bUJuUHdYdXNWazVWV3VCVm52a1JpaWoiLCJleHAiOjk5OTk5OTk5OTksImlhdCI6MCwidmMiOnsiQGNvbnRleHQiOlsiaHR0cHM6Ly93d3cudzMub3JnL25zL2NyZWRlbnRpYWxzL3YyIiwiaHR0cHM6Ly93d3cudzMub3JnL25zL2NyZWRlbnRpYWxzL2V4YW1wbGVzL3YyIl0sImlkIjoiaHR0cDovL2V4YW1wbGUuY29tL2NyZWRlbnRpYWxzLzM1MjciLCJ0eXBlIjpbIlZlcmlmaWFibGVDcmVkZW50aWFsIiwiU2VsdkNyZWRlbnRpYWwiXSwiaXNzdWVyIjoiZGlkOmlvdGE6cm1zOjB4NDJhZDU4ODMyMmU1OGIzYzA3YWEzOWU0OTQ4ZDAyMWVlMTdlY2I1NzQ3OTE1ZTllMWYzNWYwMjhkN2VjYWY5MCIsImlzc3VhbmNlRGF0ZSI6IjIwMTAtMDEtMDFUMDA6MDA6MDBaIiwibmFtZSI6IlNlbHYgQ3JlZGVudGlhbCIsImNyZWRlbnRpYWxTdWJqZWN0Ijp7ImZpcnN0X25hbWUiOiJGZXJyaXMiLCJsYXN0X25hbWUiOiJDcmFibWFuIiwiaWQiOiJkaWQ6a2V5Ono2TWtoMnllVGVSanZnY3ZOZTR5ZVF0RzhtQm5Qd1h1c1ZrNVZXdUJWbnZrUmlpaiJ9fX0.P8u516FT7-QahAIVwaVDksybMog_RB3F6cF_KgGuk6jAVWFmucWHekCXOB6UTtn0auVHCUE5B31RSsRY3H5KAA",
                "isSigned": true
            }))
            .send()
            .await
            .unwrap();
    } else if let Ok(offer_event) = serde_json::from_value::<ConnectionEvent>(payload) {
        match offer_event {
            ConnectionEvent::SIOPv2AuthorizationResponseVerified { id_token } => {
                println!("Received id_token: {:?}", id_token);
            }
            ConnectionEvent::OID4VPAuthorizationResponseVerified { vp_token } => {
                println!("Received vp_token: {:?}", vp_token);
            }
        }
    }

    "It works!".to_string()
}
