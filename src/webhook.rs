use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use crate::github::{CheckRunParams, PrCommit};
use crate::payload::{PullRequestEvent, HANDLED_ACTIONS};
use crate::{signature, validation, AppState};

pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let Some(sig) = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
    else {
        tracing::warn!("missing X-Hub-Signature-256 header");
        return StatusCode::UNAUTHORIZED;
    };
    if !signature::verify_signature(&state.config.webhook_secret, &body, sig) {
        tracing::warn!("webhook signature verification failed");
        return StatusCode::UNAUTHORIZED;
    }

    let event = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if event != "pull_request" {
        return StatusCode::OK;
    }

    let payload: PullRequestEvent = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::warn!("malformed pull_request payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };
    if !HANDLED_ACTIONS.contains(&payload.action.as_str()) {
        return StatusCode::OK;
    }
    if payload.installation.is_none() {
        tracing::warn!(
            "pull_request event without installation; is the webhook configured outside a GitHub App installation?"
        );
        return StatusCode::OK;
    }

    match process_pull_request(&state, &payload).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!("failed to process pull_request event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn process_pull_request(
    state: &AppState,
    event: &PullRequestEvent,
) -> Result<(), octocrab::Error> {
    let owner = &event.repository.owner.login;
    let repo = &event.repository.name;
    let installation = event
        .installation
        .as_ref()
        .expect("caller must filter out events without an installation")
        .id;
    let head_sha = &event.pull_request.head.sha;

    let title_params = title_check(&event.pull_request.title, head_sha);
    state
        .github
        .create_check_run(installation, owner, repo, &title_params)
        .await?;

    let commits = state
        .github
        .list_pr_commits(installation, owner, repo, event.number)
        .await?;
    let messages_params = commits_check(&commits, head_sha);
    state
        .github
        .create_check_run(installation, owner, repo, &messages_params)
        .await?;
    Ok(())
}

pub fn title_check(title: &str, head_sha: &str) -> CheckRunParams {
    match validation::validate_header(title) {
        Ok(()) => CheckRunParams {
            name: "conventional-commit-title".into(),
            head_sha: head_sha.into(),
            conclusion: "success".into(),
            title: "Conventional commit title check passed".into(),
            summary: "The pull request title meets the Conventional Commits standard.".into(),
        },
        Err(reason) => CheckRunParams {
            name: "conventional-commit-title".into(),
            head_sha: head_sha.into(),
            conclusion: "failure".into(),
            title: "Conventional commit title check failed".into(),
            summary: format!(
                "The pull request title does not meet the Conventional Commits standard: {reason}."
            ),
        },
    }
}

/// Maximum length of a commit header echoed back into a check-run summary.
/// GitHub check-run output has a 65,535-char cap; combined with
/// `MAX_LISTED_OFFENDERS` this keeps the summary far under that limit no
/// matter how many commits (or how long their headers) a pull request has.
const MAX_HEADER_DISPLAY_LEN: usize = 120;

/// Maximum number of offending commits listed individually in the summary.
const MAX_LISTED_OFFENDERS: usize = 50;

/// The GitHub `GET /repos/{owner}/{repo}/pulls/{number}/commits` endpoint
/// returns at most this many commits, regardless of pagination.
const MAX_COMMITS_PER_PULL_REQUEST: usize = 250;

/// Renders a commit header safely for inclusion inside a Markdown code span:
/// truncates to `MAX_HEADER_DISPLAY_LEN` chars (char-boundary-safe, marking
/// truncation with `…`) and strips backtick characters so the header can't
/// break out of the surrounding code span.
fn sanitize_header_for_display(header: &str) -> String {
    let truncated = if header.chars().count() > MAX_HEADER_DISPLAY_LEN {
        let mut truncated: String = header.chars().take(MAX_HEADER_DISPLAY_LEN).collect();
        truncated.push('…');
        truncated
    } else {
        header.to_string()
    };
    truncated.replace('`', "")
}

pub fn commits_check(commits: &[PrCommit], head_sha: &str) -> CheckRunParams {
    let mut failures: Vec<String> = commits
        .iter()
        .filter(|commit| commit.parents.len() < 2) // skip merge commits
        .filter_map(|commit| {
            let header = validation::first_line(&commit.commit.message);
            validation::validate_header(header).err().map(|reason| {
                let short_sha = &commit.sha[..commit.sha.len().min(7)];
                let header = sanitize_header_for_display(header);
                format!("- `{short_sha}`: `{header}` — {reason}")
            })
        })
        .collect();
    let total_failures = failures.len();

    if total_failures > MAX_LISTED_OFFENDERS {
        let remaining = total_failures - MAX_LISTED_OFFENDERS;
        failures.truncate(MAX_LISTED_OFFENDERS);
        failures.push(format!("- …and {remaining} more"));
    }

    // GitHub truncates this endpoint at 250 commits regardless of
    // pagination; when we hit that ceiling exactly, warn that earlier
    // commits on the pull request were never seen.
    let truncated_pr_note = (commits.len() == MAX_COMMITS_PER_PULL_REQUEST).then_some(
        "Note: GitHub lists at most 250 commits per pull request; earlier commits were not checked.",
    );

    if total_failures == 0 {
        let mut summary = "All commit messages meet the Conventional Commits standard.".to_string();
        if let Some(note) = truncated_pr_note {
            summary.push_str("\n\n");
            summary.push_str(note);
        }
        CheckRunParams {
            name: "conventional-commit-messages".into(),
            head_sha: head_sha.into(),
            conclusion: "success".into(),
            title: "Conventional commit messages check passed".into(),
            summary,
        }
    } else {
        let mut summary = format!(
            "The following commits do not meet the Conventional Commits standard:\n\n{}",
            failures.join("\n")
        );
        if let Some(note) = truncated_pr_note {
            summary.push_str("\n\n");
            summary.push_str(note);
        }
        CheckRunParams {
            name: "conventional-commit-messages".into(),
            head_sha: head_sha.into(),
            conclusion: "failure".into(),
            title: "Conventional commit messages check failed".into(),
            summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{CommitDetail, Parent, PrCommit};

    fn commit(sha: &str, message: &str, parent_count: usize) -> PrCommit {
        PrCommit {
            sha: sha.into(),
            commit: CommitDetail {
                message: message.into(),
            },
            parents: (0..parent_count)
                .map(|i| Parent {
                    sha: format!("parent{i}"),
                })
                .collect(),
        }
    }

    #[test]
    fn commits_check_skips_merge_commits() {
        let params = commits_check(
            &[
                commit("aaaaaaa1111", "feat: fine", 1),
                commit("bbbbbbb2222", "Merge branch 'main'", 2),
            ],
            "head",
        );
        assert_eq!(params.conclusion, "success");
    }

    #[test]
    fn commits_check_lists_offenders_with_short_sha_and_reason() {
        let params = commits_check(
            &[
                commit("aaaaaaa1111", "feat: fine", 1),
                commit("bbbbbbb2222", "fixed stuff\n\nwith a body", 1),
            ],
            "head",
        );
        assert_eq!(params.conclusion, "failure");
        assert!(params.summary.contains("`bbbbbbb`"), "{}", params.summary);
        assert!(params.summary.contains("fixed stuff"), "{}", params.summary);
        assert!(
            !params.summary.contains("with a body"),
            "{}",
            params.summary
        );
    }

    #[test]
    fn title_check_reports_reason_on_failure() {
        let params = title_check("not conventional", "head");
        assert_eq!(params.conclusion, "failure");
        assert!(params.summary.contains("Conventional Commits"));
    }

    #[test]
    fn commits_check_truncates_long_offending_header() {
        // "wip: " (a bad type) plus 200 'x's is well past the 120-char cap.
        let long_header = format!("wip: {}", "x".repeat(200));
        let params = commits_check(&[commit("aaaaaaa1111", &long_header, 1)], "head");
        assert_eq!(params.conclusion, "failure");
        // The rendered header (inside backticks) must be capped at 120 chars
        // plus the truncation marker, and must not contain the full header.
        assert!(!params.summary.contains(&long_header), "{}", params.summary);
        let expected_truncated = format!("wip: {}…", "x".repeat(115));
        assert!(
            params.summary.contains(&expected_truncated),
            "{}",
            params.summary
        );
    }

    #[test]
    fn commits_check_neutralizes_backticks_and_markdown_in_header() {
        let header = "wip: ~~x~~ **bold** [link](https://evil) `escape`";
        let params = commits_check(&[commit("aaaaaaa1111", header, 1)], "head");
        assert_eq!(params.conclusion, "failure");
        // Backticks from the header must be stripped so it can't break out
        // of the surrounding code span, but the rest of the markdown text
        // (harmless once inside a code span) is preserved verbatim.
        assert!(
            params
                .summary
                .contains("`wip: ~~x~~ **bold** [link](https://evil) escape`"),
            "{}",
            params.summary
        );
        assert!(!params.summary.contains("`escape`"), "{}", params.summary);
    }

    #[test]
    fn commits_check_caps_listed_offenders_at_fifty() {
        let commits: Vec<PrCommit> = (0..60)
            .map(|i| commit(&format!("sha{i:04}xxx"), "wip: bad", 1))
            .collect();
        let params = commits_check(&commits, "head");
        assert_eq!(params.conclusion, "failure");
        let listed_offenders = params
            .summary
            .lines()
            .filter(|line| line.starts_with("- `sha"))
            .count();
        assert_eq!(listed_offenders, 50, "{}", params.summary);
        assert!(
            params.summary.contains("- …and 10 more"),
            "{}",
            params.summary
        );
    }

    #[test]
    fn commits_check_notes_when_pull_request_hits_the_250_commit_cap() {
        let commits: Vec<PrCommit> = (0..250)
            .map(|i| commit(&format!("sha{i:04}xxx"), "feat: fine", 1))
            .collect();
        let params = commits_check(&commits, "head");
        assert_eq!(params.conclusion, "success");
        assert!(
            params.summary.contains(
                "Note: GitHub lists at most 250 commits per pull request; earlier commits were not checked."
            ),
            "{}",
            params.summary
        );
    }

    #[test]
    fn commits_check_does_not_note_cap_when_under_250_commits() {
        let params = commits_check(&[commit("aaaaaaa1111", "feat: fine", 1)], "head");
        assert_eq!(params.conclusion, "success");
        assert!(
            !params.summary.contains("Note: GitHub lists"),
            "{}",
            params.summary
        );
    }
}
