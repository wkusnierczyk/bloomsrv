# STAGE 1: Builder
# We use the official Rust image to compile the crate.
# 'slim-bookworm' is used to keep the build environment relatively small while ensuring compatibility.
FROM rust:1.85-slim-bookworm AS builder

# Set the crate name here.
ARG CRATE_NAME=bloomsrv

# Set working directory.
WORKDIR /bloomsrv

# Install necessary system dependencies for building (like OpenSSL if required by the crate)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install the crate from crates.io
# We use --root /usr/local so the binary ends up in /usr/local/bin
RUN cargo install ${CRATE_NAME} --root /usr/local

# STAGE 2: Runtime
# We use a lightweight Debian slim image for the final container.
FROM debian:bookworm-slim

# Set the crate name here.
ARG CRATE_NAME=bloomsrv

# Install runtime dependencies.
# 'ca-certificates' is often needed for HTTPS, and 'libssl3' for rust binaries linking against OpenSSL.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
# We assume the binary name matches the crate name.
# If the crate 'bloomsrv' produces a binary named differently, update the source path below.
COPY --from=builder /usr/local/bin/${CRATE_NAME} /usr/local/bin/service

# create a non-root user for security
RUN useradd -m -u 1000 -U bloomsrv
USER bloomsrv
WORKDIR /home/bloomsrv

# Expose the service port.
EXPOSE 3000

# Rust services often default to 127.0.0.1, which is unreachable from outside Docker.
# We set common environment variables to force listening on 0.0.0.0.
ENV HOST=0.0.0.0
ENV ADDRESS=0.0.0.0
ENV ROCKET_ADDRESS=0.0.0.0

# Run the service
CMD ["service"]
