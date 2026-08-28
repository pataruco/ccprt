use serde::Deserialize;

/// The pull_request webhook actions this app processes.
pub const HANDLED_ACTIONS: [&str; 4] = ["opened", "edited", "reopened", "synchronize"];

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: u64,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub installation: Option<Installation>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub title: String,
    pub head: Head,
}

#[derive(Debug, Deserialize)]
pub struct Head {
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub owner: Owner,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_all_fixtures() {
        for fixture in [
            "test/fixtures/pull_request.opened.json",
            "test/fixtures/pull_request.opened.invalid.json",
            "test/fixtures/pull_request.edited.json",
            "test/fixtures/pull_request.synchronize.json",
        ] {
            let raw = std::fs::read_to_string(fixture).unwrap();
            let event: PullRequestEvent =
                serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert!(!event.pull_request.head.sha.is_empty());
            assert_eq!(event.installation.unwrap().id, 12345);
            assert_eq!(event.number, 1);
        }
    }

    #[test]
    fn ignores_unknown_fields() {
        let raw = r#"{
            "action": "opened",
            "number": 7,
            "extra_top_level": true,
            "pull_request": {"title": "feat: x", "head": {"sha": "abc", "ref": "branch"}},
            "repository": {"name": "r", "owner": {"login": "o"}},
            "installation": {"id": 1}
        }"#;
        let event: PullRequestEvent = serde_json::from_str(raw).unwrap();
        assert_eq!(event.action, "opened");
    }

    #[test]
    fn handled_actions_are_exactly_the_spec_set() {
        assert_eq!(
            HANDLED_ACTIONS,
            ["opened", "edited", "reopened", "synchronize"]
        );
    }
}
