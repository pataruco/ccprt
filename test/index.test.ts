import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import nock from 'nock';
import { Probot, ProbotOctokit } from 'probot';
import { afterEach, beforeEach, describe, expect, test } from 'vitest';
import myProbotApp from '../src/index.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const privateKey = fs.readFileSync(
  path.join(__dirname, 'fixtures/mock-cert.pem'),
  'utf-8',
);

const validOpenedPayload = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, 'fixtures/pull_request.opened.json'),
    'utf-8',
  ),
);

const invalidOpenedPayload = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, 'fixtures/pull_request.opened.invalid.json'),
    'utf-8',
  ),
);

const validEditedPayload = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, 'fixtures/pull_request.edited.json'),
    'utf-8',
  ),
);

const validSynchronizePayload = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, 'fixtures/pull_request.synchronize.json'),
    'utf-8',
  ),
);

describe('Conventional Commit PR Title Checker', () => {
  let probot: Probot;

  beforeEach(() => {
    nock.disableNetConnect();
    probot = new Probot({
      appId: 123,
      privateKey,
      Octokit: ProbotOctokit.defaults({
        retry: { enabled: false },
        throttle: { enabled: false },
      }),
    });
    probot.load(myProbotApp);
  });

  test('creates a successful check for a valid PR title on opened event', async () => {
    const mock = nock('https://api.github.com')
      .post('/app/installations/2/access_tokens')
      .reply(200, {
        token: 'test',
        permissions: {
          checks: 'write',
          pull_requests: 'read',
        },
      })
      .post('/repos/pataruco/testing-things/check-runs', (body: unknown) => {
        expect(body).toMatchObject({
          name: 'conventional-commit',
          head_sha: 'abcdef1234567890',
          status: 'completed',
          conclusion: 'success',
          output: {
            title: 'Conventional commit check passed',
            summary:
              'The pull request title meets the conventional commit standards.',
          },
        });
        return true;
      })
      .reply(200);

    await probot.receive({
      name: 'pull_request',
      payload: validOpenedPayload,
      id: '',
    });

    expect(mock.pendingMocks()).toStrictEqual([
      'POST https://api.github.com:443/app/installations/2/access_tokens',
    ]);
  });

  test('creates a failed check for an invalid PR title on opened event', async () => {
    const mock = nock('https://api.github.com')
      .post('/app/installations/2/access_tokens')
      .reply(200, {
        token: 'test',
        permissions: {
          checks: 'write',
          pull_requests: 'read',
        },
      })
      .post('/repos/pataruco/testing-things/check-runs', (body: unknown) => {
        expect(body).toMatchObject({
          name: 'conventional-commit',
          head_sha: 'abcdef1234567890',
          status: 'completed',
          conclusion: 'failure',
          output: {
            title: 'Conventional commit check failed',
            summary:
              'The pull request title doesn not meets the conventional commit standards.',
          },
        });

        return true;
      })
      .reply(200);

    await probot.receive({
      name: 'pull_request',
      payload: invalidOpenedPayload,
      id: '',
    });

    expect(mock.pendingMocks()).toStrictEqual([
      'POST https://api.github.com:443/app/installations/2/access_tokens',
    ]);
  });

  test('creates a successful check for a valid PR title on edited event', async () => {
    const mock = nock('https://api.github.com')
      .post('/app/installations/2/access_tokens')
      .reply(200, {
        token: 'test',
        permissions: {
          checks: 'write',
          pull_requests: 'read',
        },
      })
      .post('/repos/pataruco/testing-things/check-runs', (body: unknown) => {
        expect(body).toMatchObject({
          name: 'conventional-commit',
          head_sha: 'abcdef1234567890',
          status: 'completed',
          conclusion: 'success',
          output: {
            title: 'Conventional commit check passed',
            summary:
              'The pull request title meets the conventional commit standards.',
          },
        });
        return true;
      })
      .reply(200);

    await probot.receive({
      name: 'pull_request',
      payload: validEditedPayload,
      id: '',
    });

    expect(mock.pendingMocks()).toStrictEqual([
      'POST https://api.github.com:443/app/installations/2/access_tokens',
    ]);
  });

  test('creates a successful check for a valid PR title on synchronize event', async () => {
    const mock = nock('https://api.github.com')
      .post('/app/installations/2/access_tokens')
      .reply(200, {
        token: 'test',
        permissions: {
          checks: 'write',
          pull_requests: 'read',
        },
      })
      .post('/repos/pataruco/testing-things/check-runs', (body: unknown) => {
        expect(body).toMatchObject({
          name: 'conventional-commit',
          head_sha: 'abcdef1234567890',
          status: 'completed',
          conclusion: 'success',
          output: {
            title: 'Conventional commit check passed',
            summary:
              'The pull request title meets the conventional commit standards.',
          },
        });
        return true;
      })
      .reply(200);

    await probot.receive({
      name: 'pull_request',
      payload: validSynchronizePayload,
      id: '',
    });

    expect(mock.pendingMocks()).toStrictEqual([
      'POST https://api.github.com:443/app/installations/2/access_tokens',
      'POST https://api.github.com:443/repos/pataruco/testing-things/check-runs',
    ]);
  });

  afterEach(() => {
    nock.cleanAll();
    nock.enableNetConnect();
  });
});
