#!/usr/bin/env bash

docker image pull registry.digitalocean.com/felixyeung/bins:web
docker image pull registry.digitalocean.com/felixyeung/bins:worker
docker image pull registry.digitalocean.com/felixyeung/bins:migration

docker stack deploy --with-registry-auth -c docker-compose.yaml bins
