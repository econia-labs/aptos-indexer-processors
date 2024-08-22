ARG BUILDER_VERSION=1.0.0

FROM econialabs/rust-builder:$BUILDER_VERSION AS base
WORKDIR /app

# Plan build recipe.
FROM base AS planner
COPY . .
RUN cargo chef prepare

# Install buildtime dependencies, cache runtime dependencies, then compile
# package binaries.
FROM base AS builder
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
COPY --from=planner app/recipe.json recipe.json
RUN cargo chef cook --release --locked
COPY . .
RUN cargo build --frozen --package processor --release
RUN cargo build --frozen --package indexer-metrics --release
RUN cp target/release/processor /usr/local/bin
RUN cp target/release/indexer-metrics /usr/local/bin

# Copy over package binaries, install runtime dependencies.
FROM debian:bullseye-slim
COPY --from=builder /usr/local/bin/processor /usr/local/bin
COPY --from=builder /usr/local/bin/indexer-metrics /usr/local/bin
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

# Configure logging format, executable startup script.
ENV RUST_LOG_FORMAT=json
WORKDIR /app
COPY ./start.sh .
RUN chmod u+x /app/start.sh
WORKDIR /

# Expose health check port.
EXPOSE 8084

CMD /app/start.sh
