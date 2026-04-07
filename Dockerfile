# syntax=docker/dockerfile:1
FROM rust:1.87-bookworm AS builder

WORKDIR /build
COPY . .
RUN cargo build --release -p meerkat-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/meerkat-server /usr/local/bin/meerkat-server

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/meerkat-server"]
