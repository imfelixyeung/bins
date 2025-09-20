FROM node:22.19-alpine AS nodejs
FROM rust:1.89.0-alpine AS rustlang
FROM nodejs AS base

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

# install pnpm
RUN corepack enable pnpm

WORKDIR /app

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install -g turbo@^2

COPY . .

RUN turbo prune web worker --docker



FROM base AS installer

WORKDIR /app

COPY --from=base /app/out/json .
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY --from=base /app/out/full .



FROM installer AS web-builder
WORKDIR /app/apps/web

# bypass type checking in build
ENV DATABASE_URL="postgres://postgres:postgres@db:5432/db"

ENV SKIP_SITEMAP="true"

RUN pnpm run build
RUN pnpm run build:pagefind




FROM nodejs AS web

# install bash
RUN apk update && apk add bash

WORKDIR /app/apps/web

ENV NODE_ENV=production

COPY --from=web-builder /app/apps/web/public ./apps/web/public
COPY --from=web-builder /app/apps/web/.next/standalone ./
COPY --from=web-builder /app/apps/web/.next/static ./apps/web/.next/static
COPY --from=web-builder /app/apps/web/docker-entrypoint.sh ./

EXPOSE 3000
ENV PORT=3000
ENV SKIP_SITEMAP=

RUN chmod +x ./docker-entrypoint.sh
ENTRYPOINT ["./docker-entrypoint.sh"]

CMD [ "node", "apps/web/server.js" ]


FROM rustlang AS worker-bin-builder

RUN apk update \
  && apk add pkgconfig libressl-dev musl-dev

COPY ./packages/import-csv /app/packages/import-csv
WORKDIR /app/packages/import-csv

RUN cargo update
RUN cargo build --release

CMD [ "./target/release/import-csv" ]


FROM installer AS worker-builder
WORKDIR /app/apps/worker


RUN pnpm run build



FROM nodejs AS worker 
WORKDIR /app/apps/worker

COPY --from=worker-builder /app/apps/worker/package.json .
COPY --from=worker-builder /app/apps/worker/dist/index.js ./dist/index.js
COPY --from=worker-bin-builder /app/packages/import-csv/target/release/import-csv ./bin/import-csv

CMD [ "node", "dist/index.js" ]



FROM installer AS migration-builder
WORKDIR /app/packages/database

RUN pnpm run build:migrate

RUN cat ./dist/migrate.js



FROM nodejs AS migration
WORKDIR /app/packages/database

COPY --from=migration-builder /app/packages/database/package.json .
COPY --from=migration-builder /app/packages/database/.drizzle/ ./.drizzle/
COPY --from=migration-builder /app/packages/database/dist/migrate.js ./dist/migrate.js

CMD [ "node", "dist/migrate.js" ]
