FROM rust:1.78-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p indexer-core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/indexer /usr/local/bin/indexer
ENTRYPOINT ["indexer"]
