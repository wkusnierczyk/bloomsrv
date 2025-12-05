# Configuration Variables
# You can override these from the command line: make run PORT=3000
IMAGE_NAME := bloomsrv
CONTAINER_NAME := bloomsrv
CRATE_NAME := bloomsrv
BLOOMSRV_PORT := 3000
DOCKER_PORT := 3000

.PHONY: help build run stop clean logs

# Default target prints help
help:
	@echo "Docker Lifecycle Makefile"
	@echo "-------------------------"
	@echo "Targets:"
	@echo "  make build  - Build the Docker image"
	@echo "  make run    - Run the container on port $(DOCKER_PORT)"
	@echo "  make stop   - Stop and remove the running container"
	@echo "  make clean  - Stop container and remove the docker image"
	@echo "  make logs   - View logs from the running container"

# Build the image
build:
	docker build \
		--build-arg CRATE_NAME=$(CRATE_NAME) \
		-t $(IMAGE_NAME) .

# Run the container
# Runs in detached mode (-d). Maps host PORT to container's 8080.
run:
	@echo "Starting container $(CONTAINER_NAME) on port $(DOCKER_PORT)..."
	docker run -d \
		-p $(BLOOMSRV_PORT):$(DOCKER_PORT) \
		--name $(CONTAINER_NAME) \
		$(IMAGE_NAME)

# Stop and remove the container instance
stop:
	@echo "Stopping container..."
	-docker stop $(CONTAINER_NAME)
	-docker rm $(CONTAINER_NAME)

# Clean up everything (image and container)
clean: stop
	@echo "Removing image $(IMAGE_NAME)..."
	-docker rmi $(IMAGE_NAME)

# Helper to see logs
logs:
	docker logs -f $(CONTAINER_NAME)