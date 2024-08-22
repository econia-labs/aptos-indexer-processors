ARG BUILDER_VERSION=1.0.0

# Configure base image with buildtime dependencies.
FROM econialabs/rust-builder:$BUILDER_VERSION AS base
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    curl \
    clang \
    pkg-config \
    libssl-dev \
    libdw-dev \
    libjemalloc-dev \
    libpq-dev \
    lld \
    && rm -rf /var/lib/apt/lists/*

# Plan build recipe for metrics binary.
FROM base AS metrics-planner
COPY . .
RUN cargo chef prepare --bin indexer-metrics

# Plan build recipe for processor binary.
FROM base AS processor-planner
COPY . .
RUN cargo chef prepare --bin processor

# Cache crate dependencies, then compile metrics binary.
FROM base AS metrics-builder
COPY --from=metrics-planner app/recipe.json recipe.json
RUN cargo chef cook \
    --bin indexer-metrics \
    --package indexer-metrics \
    --release
COPY . .
RUN cargo build \
    --bin indexer-metrics \
    --package indexer-metrics \
    --release
RUN cp target/release/indexer-metrics /usr/local/bin

# Cache crate dependencies, then compile processor binary.
FROM base AS processor-builder
COPY --from=processor-planner app/recipe.json recipe.json
RUN cargo chef cook \
    --bin processor \
    --package processor \
    --release
COPY . .
RUN cargo build \
    --bin processor \
    --package processor \
    --release
RUN cp target/release/processor /usr/local/bin

# Install runtime dependencies, copy over binaries.
FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install --no-install-recommends -y \
    libssl1.1 \
    ca-certificates \
    net-tools \
    tcpdump \
    iproute2 \
    netcat \
    libdw-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=metrics-builder /usr/local/bin/indexer-metrics /usr/local/bin
COPY --from=processor-builder /usr/local/bin/processor /usr/local/bin

# Configure logging format, executable startup script.
ENV RUST_LOG_FORMAT=json
WORKDIR /app
COPY ./start.sh .
RUN chmod u+x /app/start.sh
WORKDIR /

# Expose health check port.
EXPOSE 8084

CMD /app/start.sh
