# ccprt

A GitHub App that checks Conventional Commits compliance on pull requests. On
every `opened`, `edited`, `reopened`, and `synchronize` pull request action it
creates two GitHub check runs:

- **`conventional-commit-title`** — validates the pull request title.
- **`conventional-commit-messages`** — validates every non-merge commit on the
  pull request (fetched via the GitHub pulls API), re-run on each push.

Both checks validate the commit header against
[Conventional Commits](https://www.conventionalcommits.org/): it must parse as
a conventional commit, its type must be one of `feat`, `fix`, `docs`, `style`,
`refactor`, `test`, `chore`, `build`, `ci`, `perf`, `revert`, and the header
must be at most 100 characters.

## Configuration

The app reads its configuration from environment variables:

| Variable         | Required | Description                                             |
| ---------------- | -------- | -------------------------------------------------------- |
| `APP_ID`         | yes      | GitHub App ID                                            |
| `PRIVATE_KEY`    | yes      | GitHub App private key (PEM)                             |
| `WEBHOOK_SECRET` | yes      | Secret used to verify webhook payloads                   |
| `PORT`           | no       | Port to listen on (defaults to `8080`)                   |

For SOPS operations follow this [GitHub Gist](https://gist.github.com/pataruco/32d30588688c83b2d879ac06b3a5fe7e)

## Local development

```sh
# Run the test suite
cargo test

# Run the app (export the required env vars first)
export APP_ID=<app-id>
export PRIVATE_KEY=<pem-value>
export WEBHOOK_SECRET=<webhook-secret>
cargo run
```

## Docker

```sh
docker build -t ccprt .

docker run \
  -p 8080:8080 \
  -e APP_ID=<app-id> \
  -e PRIVATE_KEY=<pem-value> \
  -e WEBHOOK_SECRET=<webhook-secret> \
  ccprt
```

## Deployment

`scripts/deploy.sh` deploys the app to Cloud Run (service `ccprt`, region
`europe-west2`).

### Cutover

After deploying, point the GitHub App's webhook URL at the Cloud Run service
URL, then decommission the old Cloud Function:

```sh
gcloud functions delete ccprt --region europe-west2 --gen2
```

> **Note:** GitHub does not automatically retry failed webhook deliveries.
> If a delivery fails (for example during cutover, while the new URL is not
> yet reachable), redeliver it manually from the GitHub App's
> **Advanced → Recent Deliveries** page.
