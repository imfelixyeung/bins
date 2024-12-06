# Bins API

Get Leeds waste collection data via API

![image](https://files.felixyeung.com/cms/uploads/images/github-bins-hero.png)

## Features

- Search premises by postcode
- Get premises' scheduled waste collection dates
- Recent premises
- Suprise Me with a random premises
- API for searching premises and getting collection dates
- iCal integration

## API Reference

Available at [https://bins.felixyeung.com/docs/api](https://bins.felixyeung.com/docs/api)

## Tech Stack

- [Next.js](https://nextjs.org/) - web framework
- [TypeScript](https://www.typescriptlang.org/) - language
- [Tailwind CSS](https://tailwindcss.com/) - styling
- [Node.js](https://nodejs.org/) - backend runtime
- [PostgreSQL](https://www.postgresql.org/) - database
- [Drizzle ORM](https://orm.drizzle.team/) - database orm
- [Upstash Redis](https://upstash.com/) - rate limiting
- [Cloudflare Tunnels](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) - secure reverse proxy

## Environment Variables

To run this project, you will need to add the following environment variables to your .env file

`TUNNEL_TOKEN` - your Cloudflare Tunnel's token

## Running

### Prerequisites

- Docker
- Docker Compose

### Setup

Create your .env file as per the [Environment Variables](#environment-variables) section

Run `docker compose up` to start the project
