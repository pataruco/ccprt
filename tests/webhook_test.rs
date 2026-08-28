use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use ccprt::config::Config;
use ccprt::github::GitHubClient;
use ccprt::signature::sign;
use ccprt::{build_router, AppState};

const WEBHOOK_SECRET: &str = "test-secret";

fn test_state(mock_uri: &str) -> AppState {
    let private_key = std::fs::read_to_string("test/fixtures/mock-cert.pem").unwrap();
    let config = Config {
        app_id: 1,
        private_key: private_key.clone(),
        webhook_secret: WEBHOOK_SECRET.into(),
        port: 0,
        github_api_url: mock_uri.to_string(),
    };
    let github = GitHubClient::new(1, &private_key, mock_uri).unwrap();
    AppState {
        config: Arc::new(config),
        github,
    }
}

fn webhook_request(event: &str, body: Vec<u8>, signature: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/")
        .header("content-type", "application/json")
        .header("x-github-event", event)
        .header("x-hub-signature-256", signature)
        .body(Body::from(body))
        .unwrap()
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("test/fixtures/{name}")).unwrap()
}

async fn mock_installation_token(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/app/installations/12345/access_tokens"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "token": "test-installation-token",
            "expires_at": "2100-01-01T00:00:00Z",
            "permissions": {}
        })))
        .mount(server)
        .await;
}

fn commits_response(entries: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(entries)
}

#[tokio::test]
async fn valid_pr_gets_two_successful_check_runs() {
    let server = MockServer::start().await;
    mock_installation_token(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/pataruco/testing-things/pulls/1/commits"))
        .respond_with(commits_response(json!([
            {"sha": "aaaaaaa1111111", "commit": {"message": "feat: good commit"}, "parents": [{"sha": "p"}]},
            {"sha": "bbbbbbb2222222", "commit": {"message": "Merge branch 'main'"}, "parents": [{"sha": "p"}, {"sha": "q"}]}
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-title",
            "head_sha": "abcdef1234567890",
            "conclusion": "success"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-messages",
            "head_sha": "abcdef1234567890",
            "conclusion": "success"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 2})))
        .expect(1)
        .mount(&server)
        .await;

    let body = fixture("pull_request.opened.json");
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_title_gets_failure_check_run() {
    let server = MockServer::start().await;
    mock_installation_token(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/pataruco/testing-things/pulls/1/commits"))
        .respond_with(commits_response(json!([
            {"sha": "aaaaaaa1111111", "commit": {"message": "feat: fine"}, "parents": [{"sha": "p"}]}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-title",
            "conclusion": "failure"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-messages",
            "conclusion": "success"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 2})))
        .expect(1)
        .mount(&server)
        .await;

    let body = fixture("pull_request.opened.invalid.json");
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn synchronize_with_bad_commit_fails_messages_check_and_names_offender() {
    let server = MockServer::start().await;
    mock_installation_token(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/pataruco/testing-things/pulls/1/commits"))
        .respond_with(commits_response(json!([
            {"sha": "aaaaaaa1111111", "commit": {"message": "feat: fine"}, "parents": [{"sha": "p"}]},
            {"sha": "bbbbbbb2222222", "commit": {"message": "fixed stuff"}, "parents": [{"sha": "p"}]}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-title",
            "conclusion": "success"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 1})))
        .mount(&server)
        .await;
    // The failure summary must name the offending short sha.
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-messages",
            "conclusion": "failure"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 2})))
        .expect(1)
        .mount(&server)
        .await;

    let body = fixture("pull_request.synchronize.json");
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn bad_signature_is_rejected_without_api_calls() {
    let server = MockServer::start().await; // no mocks mounted: any call would 404 → 500
    let body = fixture("pull_request.opened.json");
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, "sha256=deadbeef"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn missing_signature_header_is_rejected() {
    let server = MockServer::start().await;
    let body = fixture("pull_request.opened.json");
    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("x-github-event", "pull_request")
        .body(Body::from(body))
        .unwrap();
    let response = build_router(test_state(&server.uri()))
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unhandled_event_and_action_return_ok_without_api_calls() {
    let server = MockServer::start().await;
    let state = test_state(&server.uri());

    let body = br#"{"action": "created"}"#.to_vec();
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(state.clone())
        .oneshot(webhook_request("issue_comment", body, &signature))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let mut closed: serde_json::Value =
        serde_json::from_slice(&fixture("pull_request.opened.json")).unwrap();
    closed["action"] = json!("closed");
    let body = serde_json::to_vec(&closed).unwrap();
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(state)
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn opened_without_installation_returns_ok_without_api_calls() {
    let server = MockServer::start().await;
    let mut payload: serde_json::Value =
        serde_json::from_slice(&fixture("pull_request.opened.json")).unwrap();
    payload.as_object_mut().unwrap().remove("installation");
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn malformed_pull_request_payload_returns_400() {
    let server = MockServer::start().await;
    let body = br#"{"action": "opened", "number": "not-a-number"}"#.to_vec();
    let signature = sign(WEBHOOK_SECRET, &body);
    let response = build_router(test_state(&server.uri()))
        .oneshot(webhook_request("pull_request", body, &signature))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn health_endpoint_responds_ok() {
    let server = MockServer::start().await;
    let response = build_router(test_state(&server.uri()))
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
