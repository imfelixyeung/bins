# Bins API

Get Leeds waste collection data via API

![image](https://files.felixyeung.com/cms/uploads/images/github-bins-hero.png)

## Features

- Search premises by postcode
- Get premises' scheduled waste collection dates
- Recent premises
- Surprise Me with a random premises
- REST API for searching premises and getting collection dates
- iCal integration for waste collection schedules
- [MCP server](https://bins.felixyeung.com/docs/mcp)
- Static site with search and offline support

## API Reference

Available at [https://bins.felixyeung.com/docs/api](https://bins.felixyeung.com/docs/api)

## Tech Stack

- [Next.js](https://nextjs.org/) (v16) - static-exported frontend with Nextra docs
- [TypeScript](https://www.typescriptlang.org/) - type-safe development
- [Tailwind CSS](https://tailwindcss.com/) - styling
- [Rust](https://www.rust-lang.org/) (Axum) - backend API and data sync
- [PostgreSQL](https://www.postgresql.org/) + PostGIS - database
- [sqlx](https://github.com/launchbadge/sqlx) - Rust database ORM/migrations
- [Traefik](https://traefik.io/) - reverse proxy and rate limiting
- [Cloudflare Tunnels](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) - secure external access

## Architecture

- **Frontend** (`apps/frontend`): Static site exported from Next.js, served by nginx. Includes the web UI, API docs (Nextra), and PWA/offline support.
- **Backend** (`apps/backend`): Rust Axum API server that handles API requests, serves sitemaps, and exposes MCP over HTTP. Also provides CLI commands for data syncing.
- **Data Sync**: Scheduled jobs that sync premises, waste collection jobs, and postcode geocoding from Leeds City Council's open data.

## Environment Variables

To run this project locally with Docker Compose, you'll need to set the following environment variables (e.g. in a `.env` file):

- `CLOUDFLARE_TUNNEL_TOKEN` - your Cloudflare Tunnel token for external access (optional if not using tunnels)

## Running

### Prerequisites

- [Docker](https://www.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)

### Setup

1. Create a `.env` file in the project root with the required environment variables as per the [Environment Variables](#environment-variables) section.
2. Run `docker compose up` to start all services.

## Development

This is a monorepo with workspaces:

- `apps/frontend` - Next.js frontend
- `apps/backend` - Rust backend

See each app's README/config for more details on local development.
