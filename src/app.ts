import type { Probot } from "probot";

const conventionalCommitTitleRegex =
	/^(feat|fix|docs|style|refactor|test|chore|build|ci|perf|revert)(\([\w\-.]+\))?(!)?:\s.{1,100}$/;

export default (app: Probot) => {
	app.on(["pull_request.opened", "pull_request.edited"], async (context) => {
		const { title, head } = context.payload.pull_request;
		const { sha } = head;

		if (conventionalCommitTitleRegex.test(title)) {
			await context.octokit.checks.create(
				context.repo({
					name: "conventional-commit",
					head_sha: sha,
					status: "completed",
					conclusion: "success",
					output: {
						title: "Conventional commit check passed",
						summary:
							"The pull request title meets the conventional commit standards.",
					},
				}),
			);
		} else {
			await context.octokit.checks.create(
				context.repo({
					name: "conventional-commit",
					head_sha: sha,
					status: "completed",
					conclusion: "failure",
					output: {
						title: "Conventional commit check failed",
						summary:
							"The pull request title doesn not meets the conventional commit standards.",
					},
				}),
			);
		}
	});
};
