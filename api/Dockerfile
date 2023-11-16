FROM node:18-alpine as base

WORKDIR /app

# install pnpm
RUN npm install -g pnpm

# install dependencies
COPY package.json pnpm-lock.yaml ./
RUN pnpm install

# copy app
COPY . .


# Development target
FROM base as dev
EXPOSE 80
CMD ["sh", "-c", "pnpm run migration:run && pnpm run dev"]


# Production target
FROM base as prod

# start app as production mode
EXPOSE 80
CMD ["sh", "-c", "pnpm run migration:run && pnpm run start"]
