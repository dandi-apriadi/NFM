# =============================================================================
# NFM Core Blockchain Node - Docker Image
# Standard: docker-expert | Target: Cross-platform (Linux, Windows, IoT)
# =============================================================================

# Stage 1: Build
FROM rust:1.77-slim-bookworm AS builder

WORKDIR /usr/src/nfm

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build actual source
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime (minimal image)
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash nfm
USER nfm
WORKDIR /home/nfm

# Copy binary
COPY --from=builder /usr/src/nfm/target/release/nfm-core-blockchain ./nfm-node

# Copy web dashboard files
COPY --chown=nfm:nfm web/ ./web/

# Expose ports: 3000 (REST API Dashboard), 9000 (P2P)
EXPOSE 3000 9000

# Data volume for blockchain persistence (sled DB)
VOLUME ["/home/nfm/nfm_main.db"]

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/api/status || exit 1

ENTRYPOINT ["./nfm-node"]
