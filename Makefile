SERVICES := web worker migration
REGISTRY := registry.digitalocean.com/felixyeung
DOCKER_STACK_NAME := bins
DOCKER_STACK_DEPLOY_FLAGS := 


help: ## Show this help
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) \
		| sed -E 's/^([a-zA-Z_-]+):.*?## (.*)/  \1\t\2/'



image-build: $(SERVICES:%=image-build--%) ## Build docker images for all services
image-build--%: ## build docker image for service $*
	docker buildx build -t bins:$* --target $* .

image-tag: $(SERVICES:%=image-tag--%) ## Tag docker images for registry
image-tag--%: ## Tag docker image $* for registry
	docker image tag bins:$* $(REGISTRY)/bins:$*



registry-push: $(SERVICES:%=registry-push--%) ## Push tagged docker images to registry
registry-push--%: ## Push tagged docker image to registry
	docker image push $(REGISTRY)/bins:$*

registry-pull: $(SERVICES:%=registry-pull--%) ## Pulls all images from registry
registry-pull--%: ## Pulls image from registry
	docker image pull $(REGISTRY)/bins:$*



deploy-local: image-build image-tag ## Build and deploys the application
	docker stack deploy $(DOCKER_STACK_DEPLOY_FLAGS) --compose-file docker-compose.yaml $(DOCKER_STACK_NAME)

deploy: registry-pull ## Pull latest images and deploys the application
	docker stack deploy $(DOCKER_STACK_DEPLOY_FLAGS) --with-registry-auth -c docker-compose.yaml $(DOCKER_STACK_NAME)


remove: ## shuts down the application
	-docker stack rm $(DOCKER_STACK_NAME) || true
