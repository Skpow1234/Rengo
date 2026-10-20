# Rengo - Multi-stage Docker build (sled runtime)

FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
    protobuf-dev \
    protoc \
    pkgconfig \
    openssl-dev \
    ca-certificates \
    musl-dev

WORKDIR /usr/src/rengo

# Manifests only — dependency cache layer
COPY Cargo.toml Cargo.lock* ./
COPY rengo-core/Cargo.toml ./rengo-core/
COPY rengo-config/Cargo.toml ./rengo-config/
COPY rengo-storage/Cargo.toml ./rengo-storage/
COPY build.rs ./

RUN if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi

# Stub sources so cargo can resolve the workspace and fetch/build deps.
# Use empty libs (not fn main) so path crates compile as libraries.
RUN mkdir -p src bin benches \
        tests/sql tests/storage tests/config \
        rengo-core/src rengo-config/src rengo-storage/src proto && \
    printf 'pub fn _docker_dep_stub() {}\n' > src/lib.rs && \
    printf 'fn main() {}\n' > bin/rengo.rs && \
    printf 'fn main() {}\n' > benches/storage.rs && \
    printf 'fn main() {}\n' > tests/sql/sql_integration.rs && \
    printf 'fn main() {}\n' > tests/storage/storage_integration.rs && \
    printf 'fn main() {}\n' > tests/config/config_integration.rs && \
    printf 'fn main() {}\n' > tests/node_integration.rs && \
    printf 'pub fn _docker_dep_stub() {}\n' > rengo-core/src/lib.rs && \
    printf 'pub fn _docker_dep_stub() {}\n' > rengo-config/src/lib.rs && \
    printf 'pub fn _docker_dep_stub() {}\n' > rengo-storage/src/lib.rs && \
    cargo build --release --bin rengo --features sled --no-default-features && \
    rm -rf src bin benches tests proto \
        rengo-core/src rengo-config/src rengo-storage/src && \
    find target/release -maxdepth 2 \( \
        -name 'librengo*' -o -name 'rengo-*' -o -name 'rengo' -o -name 'rengo.d' \
    \) -exec rm -rf {} + 2>/dev/null || true && \
    find target/release/.fingerprint -maxdepth 1 -name 'rengo*' -exec rm -rf {} + 2>/dev/null || true

# Real sources (explicit copies ensure stubs/whiteouts cannot linger)
COPY Cargo.toml Cargo.lock* build.rs ./
COPY rengo-core ./rengo-core
COPY rengo-config ./rengo-config
COPY rengo-storage ./rengo-storage
COPY src ./src
COPY bin ./bin
COPY proto ./proto
COPY benches ./benches
COPY tests ./tests
COPY config ./config

RUN cargo build --release --bin rengo --features sled --no-default-features

FROM gcr.io/distroless/cc-debian12

COPY --from=builder --chown=nonroot:nonroot /usr/src/rengo/target/release/rengo /rengo
COPY --from=builder --chown=nonroot:nonroot /usr/src/rengo/config/docker.toml /etc/rengo/config.toml

USER nonroot:nonroot
WORKDIR /

EXPOSE 5432 9090

ENTRYPOINT ["/rengo"]
CMD ["start", "--config", "/etc/rengo/config.toml"]
