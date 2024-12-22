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

# Build application
COPY . .
RUN cargo build --release --bin ${APP}

# ========================

FROM gcr.io/distroless/cc
ARG APP

ENV RUST_LOG=info

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/${APP} ./app

ENTRYPOINT ["/app/app"]
