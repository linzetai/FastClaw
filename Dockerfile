# =============================================================================
# WIP: This Dockerfile is not currently functional.
#
# XiaoLin currently ships as a Tauri desktop application (xiaolin-app).
# A standalone server binary (xiaolin CLI) is planned but not yet implemented.
# Once the CLI is available, this Dockerfile will be updated to build and
# run the server mode.
#
# For now, use the desktop application: cargo tauri dev
# =============================================================================

# ─── Build stage ─────────────────────────────────────────────────────
FROM rust:1.82-bookworm AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY extensions extensions

# TODO: Replace with `cargo build --release --bin xiaolin` when CLI crate is available
# RUN cargo build --release --bin xiaolin \
#     && strip target/release/xiaolin

# ─── Runtime stage ───────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        sqlite3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r xiaolin && useradd -r -g xiaolin -m xiaolin

WORKDIR /app

# TODO: Uncomment when CLI binary is available
# COPY --from=builder /build/target/release/xiaolin /usr/local/bin/xiaolin
COPY config/ /app/config/

RUN mkdir -p /app/data /app/logs && chown -R xiaolin:xiaolin /app

USER xiaolin

ENV RUST_LOG=info
ENV XIAOLIN_STATE_DIR=/app

EXPOSE 18789

# TODO: Uncomment when CLI binary is available
# HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
#     CMD xiaolin health || exit 1

# ENTRYPOINT ["xiaolin"]
# CMD ["serve"]
