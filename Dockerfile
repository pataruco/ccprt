FROM node:22-slim
WORKDIR /usr/src/app
COPY package.json pnpm-lock.yaml ./
RUN corepack enable
RUN corepack prepare pnpm@latest --activate
RUN pnpm i --frozen-lockfile
ENV NODE_ENV="production"
COPY . .
CMD [ "pnpm", "start" ]
