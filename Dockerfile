########################################################
# Chef image
########################################################
FROM rust:1.92 AS base

RUN cargo install cargo-chef sccache

ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache


########################################################
# Planner image
########################################################
FROM base AS planner

WORKDIR /app

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

########################################################
# Builder image
########################################################
FROM base AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release

########################################################
# Final image
########################################################
FROM debian:trixie-slim

RUN groupadd --system appuser && useradd --system --no-create-home --gid appuser appuser

WORKDIR /app
RUN chown -R appuser:appuser /app

USER appuser

COPY --from=builder /app/target/release/laterfeed /usr/local/bin/laterfeed

CMD [ "/usr/local/bin/laterfeed" ]
