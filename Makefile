help:
	@echo "make help: show this message"
	@echo "make start: start all services"
	@echo "make dev: start all services in development mode"
	@echo "make clean: stop all services"
	@echo "make upgrade: upgrade all services"

start:
	docker compose build
	docker compose up

dev:
	docker compose -f docker-compose.dev.yaml --env-file .env build
	docker compose -f docker-compose.dev.yaml --env-file .env up

clean:
	docker compose down
	docker compose -f docker-compose.dev.yaml down

upgrade:
	docker compose build
	docker compose up -d
