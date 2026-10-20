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

COPY Cargo.toml Cargo.lock* ./
COPY rengo-core/Cargo.toml ./rengo-core/
COPY rengo-config/Cargo.toml ./rengo-config/
COPY rengo-storage/Cargo.toml ./rengo-storage/

RUN if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi

# Dependency cache layer (stub sources so Cargo.toml targets resolve; benches/tests
# are excluded by .dockerignore and must exist for manifest parsing)
RUN mkdir -p src bin benches \
        tests/sql tests/storage tests/config \
        rengo-core/src rengo-config/src rengo-storage/src && \
    echo "fn main() {}" > src/lib.rs && \
    echo "fn main() {}" > bin/rengo.rs && \
    echo "fn main() {}" > benches/storage.rs && \
    echo "fn main() {}" > tests/sql/sql_integration.rs && \
    echo "fn main() {}" > tests/storage/storage_integration.rs && \
    echo "fn main() {}" > tests/config/config_integration.rs && \
    echo "fn main() {}" > tests/node_integration.rs && \
    echo "fn main() {}" > rengo-core/src/lib.rs && \
    echo "fn main() {}" > rengo-config/src/lib.rs && \
    echo "fn main() {}" > rengo-storage/src/lib.rs && \
    cargo build --release --bin rengo --features sled --no-default-features && \
    rm -rf src bin rengo-core/src rengo-config/src rengo-storage/src

COPY . .
RUN cargo build --release --bin rengo --features sled --no-default-features

FROM gcr.io/distroless/cc-debian12

COPY --from=builder --chown=nonroot:nonroot /usr/src/rengo/target/release/rengo /rengo
COPY --from=builder --chown=nonroot:nonroot /usr/src/rengo/config/docker.toml /etc/rengo/config.toml

USER nonroot:nonroot
WORKDIR /

EXPOSE 5432 9090

ENTRYPOINT ["/rengo"]
CMD ["start", "--config", "/etc/rengo/config.toml"]
