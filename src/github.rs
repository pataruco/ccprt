use octocrab::Octocrab;
use serde::Deserialize;

/// One entry from GET /repos/{owner}/{repo}/pulls/{number}/commits.
#[derive(Debug, Deserialize)]
pub struct PrCommit {
    pub sha: String,
    pub commit: CommitDetail,
    pub parents: Vec<Parent>,
}

#[derive(Debug, Deserialize)]
pub struct CommitDetail {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Parent {
    pub sha: String,
}

#[derive(Debug, Clone)]
pub struct CheckRunParams {
    pub name: String,
    pub head_sha: String,
    pub conclusion: String,
    pub title: String,
    pub summary: String,
}

const PER_PAGE: usize = 100;

#[derive(Clone)]
pub struct GitHubClient {
    app: Octocrab,
}

impl GitHubClient {
    pub fn new(app_id: u64, private_key_pem: &str, base_uri: &str) -> Result<Self, String> {
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| format!("invalid PRIVATE_KEY PEM: {e}"))?;
        let app = Octocrab::builder()
            .base_uri(base_uri)
            .map_err(|e| format!("invalid GitHub API base URI: {e}"))?
            .app(app_id.into(), key)
            .build()
            .map_err(|e| format!("failed to build GitHub client: {e}"))?;
        Ok(Self { app })
    }

    // NOTE: in octocrab 0.44.1, `Octocrab::installation()` is a synchronous
    // method returning `Result<Octocrab, octocrab::Error>` directly (it just
    // clones the client with an installation-scoped auth state; the actual
    // HTTP token exchange happens lazily on the first authenticated
    // request). The brief's draft had this as `async fn ... .await?`, which
    // does not match this octocrab version — adjusted to a plain sync call.
    fn installation(&self, installation_id: u64) -> Result<Octocrab, octocrab::Error> {
        self.app.installation(installation_id.into())
    }

    pub async fn list_pr_commits(
        &self,
        installation_id: u64,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Vec<PrCommit>, octocrab::Error> {
        let client = self.installation(installation_id)?;
        let mut commits: Vec<PrCommit> = Vec::new();
        let mut page = 1u32;
        loop {
            let batch: Vec<PrCommit> = client
                .get(
                    format!(
                        "/repos/{owner}/{repo}/pulls/{number}/commits?per_page={PER_PAGE}&page={page}"
                    ),
                    None::<&()>,
                )
                .await?;
            let batch_len = batch.len();
            commits.extend(batch);
            if batch_len < PER_PAGE {
                break;
            }
            page += 1;
        }
        Ok(commits)
    }

    pub async fn create_check_run(
        &self,
        installation_id: u64,
        owner: &str,
        repo: &str,
        params: &CheckRunParams,
    ) -> Result<(), octocrab::Error> {
        let client = self.installation(installation_id)?;
        let body = serde_json::json!({
            "name": params.name,
            "head_sha": params.head_sha,
            "status": "completed",
            "conclusion": params.conclusion,
            "output": {
                "title": params.title,
                "summary": params.summary,
            },
        });
        let _: serde_json::Value = client
            .post(format!("/repos/{owner}/{repo}/check-runs"), Some(&body))
            .await?;
        Ok(())
    }
}
