# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.72.0
ARG APP_NAME=indexer-service
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Combine package installation commands
RUN apt-get update && apt-get install -y \
    libpq-dev \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Build the application with simplified mount syntax
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

# Download and setup sink-webhook
RUN wget https://github.com/apibara/dna/releases/download/sink-webhook/v0.7.0/sink-webhook-x86_64-linux.gz && \
    gunzip sink-webhook-x86_64-linux.gz && \
    cp sink-webhook-x86_64-linux /bin/sink-webhook

# Download and setup sink-postgres
RUN wget https://github.com/apibara/dna/releases/download/sink-postgres/v0.8.0/sink-postgres-x86_64-linux.gz && \
    gunzip sink-postgres-x86_64-linux.gz && \
    cp sink-postgres-x86_64-linux /bin/sink-postgres

FROM debian:12-slim AS final

ARG UID=10001
# Combine package installation commands
RUN apt-get update && apt-get install -y \
    libpq-dev \
    procps \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries and certificates
COPY --from=build /bin/server /bin/
COPY --from=build /bin/sink-webhook /bin/
COPY --from=build /bin/sink-postgres /bin/
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

RUN chmod +x /bin/sink-webhook && \
    chmod +x /bin/sink-postgres

# Setup non-privileged user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

USER appuser

EXPOSE 8080

CMD ["/bin/server"]
