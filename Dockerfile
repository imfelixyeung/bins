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

ENV SKIP_SITEMAP="true"

RUN pnpm run build




FROM nodejs AS web

# install bash
RUN apk update && apk add bash

WORKDIR /app/apps/web

ENV NODE_ENV=production

COPY --from=web-builder /app/apps/web/public ./public
COPY --from=web-builder /app/apps/web/.next/standalone ./
COPY --from=web-builder /app/apps/web/.next/static ./apps/web/.next/static
COPY --from=web-builder /app/apps/web/docker-entrypoint.sh ./

EXPOSE 3000
ENV PORT=3000
ENV SKIP_SITEMAP=

RUN chmod +x ./docker-entrypoint.sh
ENTRYPOINT ["./docker-entrypoint.sh"]

CMD [ "node", "apps/web/server.js" ]



FROM base AS worker-builder
WORKDIR /app/apps/worker


RUN pnpm run build



FROM nodejs AS worker 
WORKDIR /app/apps/worker

COPY --from=worker-builder /app/apps/worker/package.json .
COPY --from=worker-builder /app/apps/worker/dist/index.js ./dist/index.js

CMD [ "node", "dist/index.js" ]



FROM base AS migration-builder
WORKDIR /app/packages/database

RUN pnpm run build:migrate

RUN cat ./dist/migrate.js



FROM nodejs AS migration
WORKDIR /app/packages/database

COPY --from=migration-builder /app/packages/database/package.json .
COPY --from=migration-builder /app/packages/database/.drizzle/ ./.drizzle/
COPY --from=migration-builder /app/packages/database/dist/migrate.js ./dist/migrate.js

CMD [ "node", "dist/migrate.js" ]
