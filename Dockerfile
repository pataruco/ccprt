FROM node:22-slim

# Update OS packages to reduce vulnerabilities
RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY package.json pnpm-lock.yaml ./
RUN corepack enable
RUN corepack prepare pnpm@latest --activate
RUN pnpm i --frozen-lockfile
# RUN pnpm build
ENV NODE_ENV="production"
COPY . .
CMD [ "pnpm", "start" ]
