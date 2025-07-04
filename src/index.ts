import lint from '@commitlint/lint';
import load from '@commitlint/load';
import type { LintOptions } from '@commitlint/types';
import type { Probot } from 'probot';

export default (app: Probot) => {
  app.on(['pull_request.opened', 'pull_request.edited'], async (context) => {
    const { title, head } = context.payload.pull_request;
    const { sha } = head;

    const config = await load({});
    const { rules, parserPreset } = config;
    const opts = parserPreset ? { parserOpts: parserPreset.parserOpts } : {};
    const report = await lint(title, rules, opts as LintOptions);

    if (report.valid) {
      await context.octokit.checks.create(
        context.repo({
          name: 'conventional-commit',
          head_sha: sha,
          status: 'completed',
          conclusion: 'success',
          output: {
            title: 'Conventional commit check passed',
            summary:
              'The pull request title meets the conventional commit standards.',
          },
        }),
      );
    } else {
      const summary = report.errors.map((err) => err.message).join('\n');
      await context.octokit.checks.create(
        context.repo({
          name: 'conventional-commit',
          head_sha: sha,
          status: 'completed',
          conclusion: 'failure',
          output: {
            title: 'Conventional commit check failed',
            summary,
          },
        }),
      );
    }
  });
};
