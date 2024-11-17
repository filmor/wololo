FROM rust:latest AS builder

RUN update-ca-certificates

ENV APP=wololo

# Create appuser
ENV USER=${APP}
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /app

COPY ./ .
RUN cargo build --release

FROM gcr.io/distroless/cc

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/${APP} ./app

# Use an unprivileged user.
USER ${APP}:${APP}

CMD ["/app/app"]
