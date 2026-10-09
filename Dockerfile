FROM rust:1.89-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p indexer-core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home indexer
COPY --from=builder --chown=indexer:indexer /app/target/release/indexer /usr/local/bin/indexer
USER indexer
ENTRYPOINT ["indexer"]
