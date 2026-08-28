use git_conventional::Commit;

pub const ALLOWED_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf", "revert",
];

pub const MAX_HEADER_LENGTH: usize = 100;

/// Validates a commit header (a commit message's first line, or a PR title)
/// against Conventional Commits, the allowed-type list, and the length cap.
pub fn validate_header(header: &str) -> Result<(), String> {
    if header.chars().count() > MAX_HEADER_LENGTH {
        return Err(format!(
            "header is longer than {MAX_HEADER_LENGTH} characters"
        ));
    }
    let commit = Commit::parse(header).map_err(|e| format!("not a conventional commit: {e}"))?;
    let type_ = commit.type_().as_str();
    if !ALLOWED_TYPES.contains(&type_) {
        return Err(format!(
            "type `{type_}` is not one of the allowed types ({})",
            ALLOWED_TYPES.join(", ")
        ));
    }
    Ok(())
}

/// Returns the first line of a (possibly multi-line) commit message.
pub fn first_line(message: &str) -> &str {
    message.lines().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_allowed_type() {
        for type_ in ALLOWED_TYPES {
            let header = format!("{type_}: do the thing");
            assert!(validate_header(&header).is_ok(), "rejected: {header}");
        }
    }

    #[test]
    fn accepts_scope_breaking_bang_and_scoped_bang() {
        for header in [
            "feat(api): add endpoint",
            "feat!: breaking change",
            "feat(api)!: breaking scoped change",
            "fix(deps-dev.sub): dotted and dashed scope",
        ] {
            assert!(validate_header(header).is_ok(), "rejected: {header}");
        }
    }

    #[test]
    fn rejects_disallowed_type_with_reason() {
        let err = validate_header("wip: still cooking").unwrap_err();
        assert!(err.contains("wip"), "error was: {err}");
    }

    #[test]
    fn rejects_non_conventional_headers() {
        for header in [
            "Add new feature",
            "feat add feature",
            "feat:",
            "feat: ",
            ": no type",
            "",
        ] {
            assert!(validate_header(header).is_err(), "accepted: {header}");
        }
    }

    #[test]
    fn enforces_100_char_header_limit() {
        let padding_99 = "x".repeat(99 - "feat: ".len());
        assert!(validate_header(&format!("feat: {padding_99}")).is_ok());
        let padding_101 = "x".repeat(101 - "feat: ".len());
        let err = validate_header(&format!("feat: {padding_101}")).unwrap_err();
        assert!(err.contains("100"), "error was: {err}");
    }

    #[test]
    fn first_line_takes_header_only() {
        assert_eq!(first_line("feat: thing\n\nlong body"), "feat: thing");
        assert_eq!(first_line("feat: thing"), "feat: thing");
        assert_eq!(first_line(""), "");
    }
}
