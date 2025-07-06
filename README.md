# ccprt

A GitHub App built with [Probot](https://github.com/probot/probot) that checks pr titles follows conventional commits conventions

## Setup

```sh
# Install dependencies
pnpm install

# Run the bot
pnpm start
```

For SOPS operations follow this [GitHub Gist](https://gist.github.com/pataruco/32d30588688c83b2d879ac06b3a5fe7e)

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
