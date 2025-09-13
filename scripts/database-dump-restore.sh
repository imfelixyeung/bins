docker exec -it CONTAINER pg_dump -U postgres db > pg_dump.backup
cat pg_dump.backup | docker exec -i CONTAINER psql -U postgres db
