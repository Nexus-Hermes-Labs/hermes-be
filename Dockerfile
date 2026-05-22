# ─── Stage 1: Chef base ─────────────────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler libprotobuf-dev curl \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ─── Stage 2: Dependency planner ────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ─── Stage 3: Builder ───────────────────────────────────────────────────────
FROM chef AS builder
ARG SERVICE_NAME
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin ${SERVICE_NAME}

# ─── Stage 4: Runtime ───────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
ARG SERVICE_NAME
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/${SERVICE_NAME} /app/service
COPY --from=builder /app/config /app/config
ENTRYPOINT ["/app/service"]
