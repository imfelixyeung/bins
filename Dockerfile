FROM node:22.6.0-alpine AS base

# install pnpm
RUN npm install -g pnpm

# set working directory
WORKDIR /app

COPY . .

RUN pnpm install --frozen-lockfile



FROM base AS web

WORKDIR /app/apps/web

EXPOSE 3000

RUN pnpm run build

CMD ["pnpm", "run", "start"]



FROM base AS worker

WORKDIR /app/apps/worker

CMD ["pnpm", "run", "start"]
