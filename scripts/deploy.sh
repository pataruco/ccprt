#!/usr/bin/env bash
set -euo pipefail

# Secrets come from the environment — never hardcode them here.
# Run via sops so they are injected for this process only:
#
#   sops exec-env secrets-encrypted.env 'bash scripts/deploy.sh'
#
: "${APP_ID:?APP_ID is not set — run: sops exec-env secrets-encrypted.env 'bash scripts/deploy.sh'}"
: "${WEBHOOK_SECRET:?WEBHOOK_SECRET is not set — run via sops exec-env (see comment above)}"
: "${PRIVATE_KEY:?PRIVATE_KEY is not set — run via sops exec-env (see comment above)}"

gcloud run deploy ccprt \
  --source . \
  --clear-base-image \
  --region europe-west2 \
  --allow-unauthenticated \
  --set-env-vars "APP_ID=${APP_ID},WEBHOOK_SECRET=${WEBHOOK_SECRET}" \
  --set-env-vars "^@^PRIVATE_KEY=${PRIVATE_KEY}"
