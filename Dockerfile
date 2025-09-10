FROM node:22.19-alpine AS base

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



FROM base AS worker-builder

WORKDIR /app/apps/worker

RUN pnpm run build



FROM node:22.19-alpine as worker 

COPY --from=worker-builder /app/apps/worker/package.json .
COPY --from=worker-builder /app/apps/worker/dist/index.js ./dist/index.js

CMD [ "sh", "-c", "node dist/index.js" ]



FROM base as migration-builder

WORKDIR /app/packages/database

RUN pnpm run build:migrate

RUN cat ./dist/migrate.js



FROM node:22.19-alpine as migration
WORKDIR /app/packages/database

COPY --from=migration-builder /app/packages/database/package.json .
COPY --from=migration-builder /app/packages/database/.drizzle/ ./.drizzle/
COPY --from=migration-builder /app/packages/database/dist/migrate.js ./dist/migrate.js

CMD [ "sh", "-c", "node dist/migrate.js" ]
