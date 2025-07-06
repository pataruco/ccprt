# ccprt

A GitHub App built with [Probot](https://github.com/probot/probot) that checks pr titles follows conventional commits conventions

## Setup

```sh
# Install dependencies
pnpm install

# Run the bot
pnpm start
```

## Tooling

### Tests

```sh
pnpm test
```

### Lint

```sh
pnpm lint
```

## Docker

```sh
# 1. Build container
docker build -t ccprt .

# 2. Start container
docker run -e APP_ID=<app-id> -e PRIVATE_KEY=<pem-value> ccprt

docker run \
-e APP_ID=<app-id> \
-e PRIVATE_KEY=<pem-value>
ccprt
```
