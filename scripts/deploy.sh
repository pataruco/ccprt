export WEBHOOK_PROXY_URL=''
export APP_ID=''
export PRIVATE_KEY=''
export WEBHOOK_SECRET=''
export GITHUB_CLIENT_ID=''
export GITHUB_CLIENT_SECRET=''

gcloud functions deploy ccprt \
--gen2 \
--region europe-west2 \
--runtime nodejs22 \
--allow-unauthenticated \
--trigger-http \
--entry-point probotApp \
--set-env-vars APP_ID="$APP_ID",PRIVATE_KEY="$PRIVATE_KEY",WEBHOOK_SECRET="$WEBHOOK_SECRET",GITHUB_CLIENT_ID="$GITHUB_CLIENT_ID",GITHUB_CLIENT_SECRET="$GITHUB_CLIENT_SECRET"
