#!/usr/bin/env bash
set -euo pipefail

# Fill these from sops-decrypted secrets before running (Pedro runs this).
export APP_ID=''
export PRIVATE_KEY=''
export WEBHOOK_SECRET=''

gcloud run deploy ccprt \
  --source . \
  --region europe-west2 \
  --allow-unauthenticated \
  --set-env-vars "APP_ID=${APP_ID},WEBHOOK_SECRET=${WEBHOOK_SECRET}" \
  --set-env-vars "^@^PRIVATE_KEY=${PRIVATE_KEY}"
