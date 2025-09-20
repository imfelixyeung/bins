REGISTRY := registry.digitalocean.com/felixyeung
DOCKER_STACK_NAME := bins
DOCKER_STACK_DEPLOY_FLAGS := 

help: ## Show this help
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) \
		| sed -E 's/^([a-zA-Z_-]+):.*?## (.*)/  \1\t\2/'

build: ## Build docker images for all services
	docker buildx build -t bins-web --target web .
	docker buildx build -t bins-worker --target worker .
	docker buildx build -t bins-migration --target migration .

tag: ## Tag docker images for registry
	docker image tag bins-web $(REGISTRY)/bins:web
	docker image tag bins-worker $(REGISTRY)/bins:worker
	docker image tag bins-migration $(REGISTRY)/bins:migration

registry-push: ## Push tagged docker images to registry
	docker image push $(REGISTRY)/bins:web
	docker image push $(REGISTRY)/bins:worker
	docker image push $(REGISTRY)/bins:migration

registry-pull: ## Pulls all images from registry
	docker image pull $(REGISTRY)/bins:web
	docker image pull $(REGISTRY)/bins:worker
	docker image pull $(REGISTRY)/bins:migration

deploy-local: build tag ## Build and deploys the application
	docker stack deploy $(DOCKER_STACK_DEPLOY_FLAGS) --compose-file docker-compose.yaml $(DOCKER_STACK_NAME)

deploy: registry-pull ## Pull latest images and deploys the application
	docker stack deploy $(DOCKER_STACK_DEPLOY_FLAGS) --with-registry-auth -c docker-compose.yaml $(DOCKER_STACK_NAME)