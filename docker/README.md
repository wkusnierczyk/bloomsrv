# `bloomsrv` Docker container

This project contains a `Dockerfile` to build a Docker image encapsulating the `bloomsrv` service and expose it in a running container.

## Contents

* [Prerequisites](#prerequisites)
* [Build the image](#build-the-image)
* [Run the container](#run-the-container)
* [Stop the container](#stop-the-container)
* [Clean up](#clean-up)
* [Print logs](#print-logs)
* [Testing](#testing)

## Prerequisites

* [Docker](https://www.docker.com/): to build and run the container.
* [Make](https://www.gnu.org/software/make/): to use build system commands instead of invoking docker commands manually. 

The included `Makefile` provides targets to build, run, stop, and clean up the container, with suitable deafaults.
To customize the build process, override the build arguments in the `Makefile` while executing `make`. 

**Note**
* Check the `Makefile` for more details.
No detailed documentation on customizing the `make` build is provided as of this version; further will be provided at a later time.


## Build the image

Run the following commands in the directory containing the Dockerfile.

```bash
# Specify the image name
export IMAGE_NAME="bloomsrv"

# Option 1: Use docker
docker build -t "${IMAGE_NAME}" .

# Option 2: Use make
make clean build
```

**Note**
* By default, the service will listen on `http://127.0.0.1:3000`.
* To override the settings, set the build arguments `BLOOMSRV_HOST` and `BLOOMSRV_PORT`.

```bash
# Specify the host and port to listen on
export BLOOMSRV_HOST=<host>
export BLOOMSRV_PORT=<port>

# Option 1: Use docker
docker build -t "${IMAGE_NAME}" \
             --build-arg BLOOMSRV_HOST="${BLOOMSRV_HOST}" \ 
             --build-arg BLOOMSRV_PORT="${BLOOMSRV_PORT}" \
             .
```

## Run the container

Run the following commands to start the container.

**Note**
* If the image has not been built locally, as in the preceding step, an attempt will be made to pull the image will be pulled from Docker Hub instead.

```bash
# Specify the port mapping
# This is the port bloomsrv listens on within the container; 
# must match the value of BLOOMSRV_PORT used in the build step
export CONTAINER_PORT=<port>
# This is the port the host listens on for traffic to the container  
export HOST_PORT=<port>

# Specify the name for the running container
export SERVICE_NAME="bloomsrv"

# Option 1: Use docker
docker run -p "${HOST_PORT}":"${CONTAINER_PORT}" \
           --name "${SERVICE_NAME}" \
           "${IMAGE_NAME}"
           
# Option 2: Use make
make run
```

## Stop the container

Use the following commands to stop the container.

```bash
# Option 1: Use docker
docker stop "${SERVICE_NAME}"
docker rm "${SERVICE_NAME}"

# Option 2: Use make
make stop
```

## Clean up

Use the following commands to remove the image.

```bash
# Option 1: Use docker
docker rmi "${IMAGE_NAME}"

# Option 2: Use make
make clean
```

## Print logs

Use the follwoing commands to print the logs of a running container.

```bash
# Option 1: Use docker
docker logs "${SERVICE_NAME}"     # just print the logs
docker logs -f "${SERVICE_NAME}"  # print and listen for further updates

# Option 2: Use make
make logs                         # print and listen for further updates (-f implied)
```

## Testing

No testing is provided for the Docker image.