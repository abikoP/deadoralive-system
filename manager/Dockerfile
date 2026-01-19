# ===========================================
# Stage 1: Builder
# ===========================================
FROM rust:1.85-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create dummy source files to build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/seed_user.rs && \
    echo "fn main() {}" > src/bin/test_password.rs && \
    mkdir -p migration/src && \
    echo "pub use sea_orm_migration::prelude::*; pub struct Migrator; #[async_trait::async_trait] impl MigratorTrait for Migrator { fn migrations() -> Vec<Box<dyn MigrationTrait>> { vec![] } }" > migration/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src migration/src target/release/.fingerprint/kiro_test-*

# Copy actual source code
COPY src ./src
COPY migration ./migration
COPY config ./config
COPY assets ./assets

# Build the application
RUN cargo build --release

# ===========================================
# Stage 2: Runtime
# ===========================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies and Docker CLI
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    gnupg \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI (for Telegraf control)
RUN install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    chmod a+r /etc/apt/keyrings/docker.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
    $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/kiro_test /app/kiro_test

# Copy configuration and assets
COPY --from=builder /app/config /app/config
COPY --from=builder /app/assets /app/assets

# Create directories for data and config
RUN mkdir -p /data /config

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///data/db.sqlite
ENV CONFIG_PATH=/config/telegraf.conf

# Expose port
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Run the application
CMD ["/app/kiro_test", "start"]
