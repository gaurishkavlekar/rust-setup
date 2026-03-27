# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1.76-slim-bookworm AS builder

WORKDIR /app

# Install system deps for sqlx (needs OpenSSL)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies layer
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build the actual app
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-api /app/rust-api
COPY --from=builder /app/migrations /app/migrations

# Non-root user
RUN useradd -r -s /bin/false appuser && chown -R appuser /app
USER appuser

EXPOSE 8080

CMD ["/app/rust-api"]
