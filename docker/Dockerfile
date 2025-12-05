# === STAGE 1: Builder ===
# Use the official Rust image to compile the crate.
# 'slim-bookworm' is used to keep the build environment relatively small while ensuring compatibility.
FROM rust:1.85-slim-bookworm AS builder

# Set the crate name here.
ARG CRATE_NAME=bloomsrv

# Set the installation root directory
ARG INSTALL_ROOT=/usr/local

# Set working directory.
WORKDIR /bloomsrv

# Install necessary system dependencies for building (like OpenSSL if required by the crate).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install the crate from crates.io.
# If --root ${INSTALL_ROOT} is used, the binary ends up in ${INSTALL_ROOT}/bin.
RUN cargo install ${CRATE_NAME} --root ${INSTALL_ROOT}

# === STAGE 2: Runtime ===
# Use a lightweight Debian slim image for the final container.
FROM debian:bookworm-slim

# Set the crate.
ARG CRATE_NAME=bloomsrv

# Set the service name; by default, the same as the crate name.
ARG SERVICE_NAME=bloomsrv

# Set the name for non-root user to run the service
ARG SERVICE_USER=bloomsrv

# Specify the host and port for the service to listen on/to.
ARG SERVICE_HOST=0.0.0.0
ARG SERVICE_PORT=3000

# Set the installation root directory
ARG INSTALL_ROOT=/usr/local

# Install runtime dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage.
COPY --from=builder ${INSTALL_ROOT}/bin/${CRATE_NAME} ${INSTALL_ROOT}/bin/${SERVICE_NAME}

# Create a non-root user for security.
RUN useradd -m -u 1000 -U ${SERVICE_USER}
USER ${SERVICE_USER}
WORKDIR /home/${SERVICE_USER}

# Expose the service port.
EXPOSE ${SERVICE_PORT}

# Run the service
CMD ["service", "--host", "${SERVICE_HOST}", "--port", "${SERVICE_PORT}"]
