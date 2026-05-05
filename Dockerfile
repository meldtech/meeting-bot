# syntax=docker/dockerfile:1

FROM rust:bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/kair-meeting-bot /app/kair-meeting-bot
ENV HOST=0.0.0.0
EXPOSE 8090
USER nobody
CMD ["/app/kair-meeting-bot"]
