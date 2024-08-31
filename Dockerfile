FROM node:22.6.0-alpine AS base

# install pnpm
RUN npm install -g pnpm

# set working directory
WORKDIR /app

COPY . .

RUN pnpm install --frozen-lockfile



FROM base AS web

WORKDIR /app/apps/web

# bypass type checking in build
ENV DATABASE_URL="postgres://postgres:postgres@db:5432/db"

EXPOSE 3000

RUN pnpm run build

CMD ["pnpm", "run", "start"]



FROM base AS worker

# needed to import csv files into postgres
RUN apk add postgresql-client
# sed installed doesn't replace hex nulls
RUN apk add sed

WORKDIR /app/apps/worker

RUN pnpm run build

CMD ["pnpm", "run", "start"]



FROM base as migration
CMD [ "sh", "-c", "pnpm --filter @repo/database run migrate" ]
