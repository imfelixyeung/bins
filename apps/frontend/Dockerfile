# syntax=docker/dockerfile:1

# Build context is the repository root, not this directory: apps/frontend is a bun
# workspace member, so its dependencies install from the root lockfile, and
# next.config.ts points turbopack at the root as well.
FROM oven/bun:1.4.2 AS builder

WORKDIR /repo

ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production

# Where the exported pages say they live, baked into the sitemap and metadata at
# build time. Nothing reads the environment once the site is static.
ARG BASE_URL=https://bins.felixyeung.com
ENV BASE_URL=${BASE_URL}

# Install from the workspace lockfile on its own layer, so editing app sources
# does not reinstall dependencies. apps/frontend/package.json has to be copied
# too, since the lockfile describes it as a workspace.
COPY package.json bun.lock .npmrc ./
COPY apps/frontend/package.json ./apps/frontend/package.json
RUN bun install --frozen-lockfile

COPY apps/frontend ./apps/frontend

# output: "export" in next.config.ts writes the whole site to apps/frontend/out.
WORKDIR /repo/apps/frontend
RUN bun run build


# nginx serves the exported files, so the runtime image has no node in it at all.
FROM nginx:1.29-alpine AS runtime

COPY apps/frontend/docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /repo/apps/frontend/out /usr/share/nginx/html

# Unprivileged, so the listener does not have to be port 80.
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1