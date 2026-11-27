# syntax=docker/dockerfile:1
# Build:  docker build -t tuita .
# Run:    docker run --rm --network host -e DATABASE_URL=postgres://tuita@localhost/tuita tuita

FROM rust:1.99-slim AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config make perl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY crates ./crates
COPY vendor ./vendor
# Cache mounts keep the registry and target/ between builds, so source edits do not rebuild
# dependencies, and the multi-GB target/ never becomes an image layer. topcoat-cli (the asset
# bundler) is pinned to the topcoat crate version and installed into the cached CARGO_HOME.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo install --locked --version 0.12.1 --target-dir /src/target/topcoat-cli topcoat-cli \
    && SQLX_OFFLINE=true cargo build --release --locked -p tuita \
    && SQLX_OFFLINE=true topcoat asset bundle --release --package tuita \
    && cp target/release/tuita /usr/local/bin/tuita \
    && cp -r target/release/assets /usr/local/bin/assets \
    && mkdir -p /data/media

# distroless/cc has glibc and libgcc, which the default Rust target needs. No shell, no package manager.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /usr/local/bin/tuita /usr/local/bin/tuita
# The asset bundle must sit beside the executable: `tuita_web::Assets::next_to_binary()`.
COPY --from=build /usr/local/bin/assets /usr/local/bin/assets
# Uploaded media live here; distroless has no shell to create a writable dir, so copy an empty one.
COPY --from=build --chown=nonroot:nonroot /data /data
# The server binds loopback by default; a container must listen on all interfaces.
ENV DATA_DIR=/data \
    BIND_ADDR=0.0.0.0:8080 \
    LOG_FORMAT=json
VOLUME /data
EXPOSE 8080
USER nonroot
# No HEALTHCHECK: distroless has no shell, curl or wget to run one, and Docker's HEALTHCHECK
# needs one. Probe `GET /healthz` (liveness) and `GET /readyz` (readiness) from the orchestrator
# instead (Kubernetes httpGet probes, Nomad/Consul HTTP checks, a load balancer health check).
ENTRYPOINT ["/usr/local/bin/tuita", "serve"]
