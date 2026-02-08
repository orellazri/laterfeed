FROM rust:1.92-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release && rm -rf src

COPY src ./src
COPY .sqlx ./.sqlx
COPY migrations ./migrations

RUN touch src/main.rs src/lib.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/laterfeed ./laterfeed

EXPOSE 8000

ENTRYPOINT ["./laterfeed"]
