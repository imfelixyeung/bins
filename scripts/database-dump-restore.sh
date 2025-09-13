#!/usr/bin/env bash

# script to dump the contents of a local environment database into a production instance
# when the prod instance doesn't have enough memory to parse all the csv

mkdir -p temp

echo 'creating ./temp/pg_dump.backup'
docker exec -it $(docker ps --filter "NAME=bins_db" -q) pg_dump --clean -U postgres db > ./temp/pg_dump.backup

echo 'uploading ./temp/pg_dump.backup to production server'
rsync -av --progress ./temp/pg_dump.backup bins:~/

echo 'restoring the database'
ssh bins -t 'cat ~/pg_dump.backup | docker exec -i $(docker ps --filter "NAME=bins_db" -q) psql -U postgres db'
