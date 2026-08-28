use ccprt::github::{CheckRunParams, GitHubClient};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client(base_uri: &str) -> GitHubClient {
    let key = std::fs::read_to_string("test/fixtures/mock-cert.pem").unwrap();
    GitHubClient::new(1, &key, base_uri).unwrap()
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

#[tokio::test]
async fn lists_pr_commits_with_parents() {
    let server = MockServer::start().await;
    mock_installation_token(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/pataruco/testing-things/pulls/1/commits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "sha": "aaaaaaa1111111",
                "commit": {"message": "feat: first\n\nbody"},
                "parents": [{"sha": "p1"}]
            },
            {
                "sha": "bbbbbbb2222222",
                "commit": {"message": "Merge branch 'main' into feature"},
                "parents": [{"sha": "p1"}, {"sha": "p2"}]
            }
        ])))
        .mount(&server)
        .await;

    let commits = test_client(&server.uri())
        .list_pr_commits(12345, "pataruco", "testing-things", 1)
        .await
        .unwrap();

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].commit.message, "feat: first\n\nbody");
    assert_eq!(commits[1].parents.len(), 2);
}

#[tokio::test]
async fn creates_check_run_with_expected_body() {
    let server = MockServer::start().await;
    mock_installation_token(&server).await;
    Mock::given(method("POST"))
        .and(path("/repos/pataruco/testing-things/check-runs"))
        .and(body_partial_json(json!({
            "name": "conventional-commit-title",
            "head_sha": "abcdef1234567890",
            "status": "completed",
            "conclusion": "success",
            "output": {"title": "Conventional commit title check passed"}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .create_check_run(
            12345,
            "pataruco",
            "testing-things",
            &CheckRunParams {
                name: "conventional-commit-title".into(),
                head_sha: "abcdef1234567890".into(),
                conclusion: "success".into(),
                title: "Conventional commit title check passed".into(),
                summary: "The pull request title meets the Conventional Commits standard.".into(),
            },
        )
        .await
        .unwrap();
    // MockServer verifies .expect(1) on drop.
}
