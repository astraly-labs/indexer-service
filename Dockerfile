# syntax=docker/dockerfile-upstream:master

# Comments are provided throughout this file to help you get started.
# If you need more help, visit the Dockerfile reference guide at
# https://docs.docker.com/engine/reference/builder/

################################################################################
# Create a stage for building the application.

ARG RUST_VERSION=1.72.0
ARG APP_NAME=indexer-service
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app



RUN apt update
RUN apt install -y libpq-dev

# Install ca-certificates needed for AWS sdk
RUN apt-get install -y --no-install-recommends ca-certificates
RUN apt-get install -y --no-install-recommends wget



# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<-EOF
    set -e
    ls -la
    cargo build --locked --release
    cp ./target/release/$APP_NAME /bin/server
EOF

# Download sink-webhook from the Github release
RUN wget https://github.com/apibara/dna/releases/download/sink-webhook/v0.7.0/sink-webhook-x86_64-linux.gz
RUN gunzip sink-webhook-x86_64-linux.gz
RUN cp sink-webhook-x86_64-linux /bin/sink-webhook

# Download sink-postgres from the Github release
RUN wget https://github.com/apibara/dna/releases/download/sink-postgres/v0.8.0/sink-postgres-x86_64-linux.gz
RUN gunzip sink-postgres-x86_64-linux.gz
RUN cp sink-postgres-x86_64-linux /bin/sink-postgres

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the debian bullseye image as the foundation for running the app.
# By specifying the "bullseye-slim" tag, it will also use whatever happens to be the
# most recent version of that tag when you build your Dockerfile. If
# reproducability is important, consider using a digest
# (e.g., debian@sha256:ac707220fbd7b67fc19b112cee8170b41a9e97f703f588b2cdbbcdcecdd8af57).
FROM debian:12-slim AS final

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
ARG UID=10001
RUN apt update
RUN apt install -y libpq-dev
RUN apt-get install -y procps openssl

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/
# Copy all the app binaries
COPY --from=build /bin/sink-webhook /bin/
RUN chmod +x /bin/sink-webhook
COPY --from=build /bin/sink-postgres /bin/
RUN chmod +x /bin/sink-postgres
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Expose the port that the application listens on.
EXPOSE 8080

# What the container should run when it is started.
CMD ["/bin/server"]
