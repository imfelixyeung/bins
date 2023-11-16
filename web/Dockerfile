FROM node:18-alpine as base

WORKDIR /app

# install pnpm
RUN npm install -g pnpm

# install dependencies
COPY package.json pnpm-lock.yaml ./
RUN pnpm install

# copy source code
COPY . .


# Development target
FROM base as dev
EXPOSE 80
CMD [ "pnpm", "dev" ]



# Production target
FROM base as prod

# build app
RUN pnpm build

# serve app
EXPOSE 80
CMD [ "pnpm", "start" ]
