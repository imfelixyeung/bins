#!/usr/bin/env bash

docker buildx build -t bins-web --target web .
docker buildx build -t bins-worker --target worker .
docker buildx build -t bins-migration --target migration .

docker image tag bins-web registry.digitalocean.com/felixyeung/bins:web
docker image tag bins-worker registry.digitalocean.com/felixyeung/bins:worker
docker image tag bins-migration registry.digitalocean.com/felixyeung/bins:migration

docker image push registry.digitalocean.com/felixyeung/bins:web
docker image push registry.digitalocean.com/felixyeung/bins:worker
docker image push registry.digitalocean.com/felixyeung/bins:migration
