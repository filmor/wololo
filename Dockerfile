ARG APP=wololo

FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ========================

FROM chef AS builder 
ARG APP

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Install the bundler separately so source-only changes reuse this layer.
RUN cargo install topcoat-cli --version 0.5.0 --locked

# Build the release binary and its Topcoat asset bundle.
COPY . .
RUN cargo build --release --locked --bin ${APP} \
    && topcoat asset bundle --release --bin ${APP} --out /app/assets

# ========================

FROM gcr.io/distroless/cc
ARG APP

ENV RUST_LOG=info
ENV WOLOLO_ASSET_DIR=/app/assets

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/${APP} ./app
COPY --from=builder /app/assets ./assets

ENTRYPOINT ["/app/app"]
