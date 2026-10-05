# syntax=docker/dockerfile:1

# Build context is ./apps/backend, which is a self-contained crate. Keep this in
# step with rust-toolchain.toml; sqlx 0.9 needs rustc >= 1.94.
FROM rust:1.99-bookworm AS builder

WORKDIR /app

# Build dependencies against a stub crate first so the (slow) dependency graph
# is cached and only the workspace needs rebuilding when sources change.
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-target,target=/app/target \
    mkdir -p src \
 && echo 'fn main() {}' > src/main.rs \
 && : > src/lib.rs \
 && cargo build --release

# sqlx::migrate!() embeds these at compile time, so they must be present for the
# real build.
COPY migrations ./migrations
COPY src ./src

RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-target,target=/app/target \
    touch src/main.rs src/lib.rs \
 && cargo build --release \
 && cp /app/target/release/backend /usr/local/bin/backend \
 && strip /usr/local/bin/backend


FROM debian:bookworm-slim AS runtime

# ca-certificates is needed to verify the TLS certificate of the upstream open
# data host when syncing.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --uid 1001 backend

COPY --from=builder /usr/local/bin/backend /usr/local/bin/backend

USER backend
WORKDIR /home/backend

EXPOSE 3000

ENV RUST_LOG=backend=info
ENV PORT=3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${PORT}/" || exit 1

CMD ["backend", "serve"]


# Scheduled sync service. crond runs as root purely so it can drop to the
# `backend` user per job; the jobs themselves stay unprivileged. Which sync runs,
# and at what time, comes from the environment via the entrypoint, so one image
# serves both services.
FROM runtime AS cron

USER root
RUN apt-get update \
 && apt-get install -y --no-install-recommends cron \
 && rm -rf /var/lib/apt/lists/*

COPY docker/cron/entrypoint.sh /usr/local/bin/cron-entrypoint
RUN chmod 0755 /usr/local/bin/cron-entrypoint

# These containers run cron, not the API, so the healthcheck inherited from the
# runtime stage would always fail.
HEALTHCHECK NONE

ENTRYPOINT ["cron-entrypoint"]