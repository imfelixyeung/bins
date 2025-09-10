FROM node:22.19-alpine AS nodejs
FROM nodejs AS base

# install pnpm
RUN corepack enable pnpm

# set working directory
WORKDIR /app

COPY . .

RUN pnpm install --frozen-lockfile



FROM base AS web-builder
WORKDIR /app/apps/web

# bypass type checking in build
ENV DATABASE_URL="postgres://postgres:postgres@db:5432/db"


RUN pnpm run build




FROM nodejs as web
WORKDIR /app/apps/web

ENV NODE_ENV=production

COPY --from=web-builder /app/apps/web/public ./public
COPY --from=web-builder /app/apps/web/.next/standalone ./
COPY --from=web-builder /app/apps/web/.next/static ./apps/web/.next/static

EXPOSE 3000
ENV PORT=3000

CMD [ "sh", "-c", "node apps/web/server.js" ]



FROM base AS worker-builder
WORKDIR /app/apps/worker


RUN pnpm run build



FROM nodejs as worker 
WORKDIR /app/apps/worker

COPY --from=worker-builder /app/apps/worker/package.json .
COPY --from=worker-builder /app/apps/worker/dist/index.js ./dist/index.js

CMD [ "sh", "-c", "node dist/index.js" ]



FROM base as migration-builder
WORKDIR /app/packages/database

RUN pnpm run build:migrate

RUN cat ./dist/migrate.js



FROM nodejs as migration
WORKDIR /app/packages/database

COPY --from=migration-builder /app/packages/database/package.json .
COPY --from=migration-builder /app/packages/database/.drizzle/ ./.drizzle/
COPY --from=migration-builder /app/packages/database/dist/migrate.js ./dist/migrate.js

CMD [ "sh", "-c", "node dist/migrate.js" ]
