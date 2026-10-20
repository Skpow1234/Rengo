# ShardForge - Multi-stage Docker build (sled runtime)

FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
    protobuf-dev \
    protoc \
    pkgconfig \
    openssl-dev \
    ca-certificates \
    musl-dev

WORKDIR /usr/src/shardforge

COPY Cargo.toml Cargo.lock* ./
COPY shardforge-core/Cargo.toml ./shardforge-core/
COPY shardforge-config/Cargo.toml ./shardforge-config/
COPY shardforge-storage/Cargo.toml ./shardforge-storage/

RUN if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi

# Dependency cache layer
RUN mkdir -p src bin shardforge-core/src shardforge-config/src shardforge-storage/src && \
    echo "fn main() {}" > src/lib.rs && \
    echo "fn main() {}" > bin/shardforge.rs && \
    echo "fn main() {}" > shardforge-core/src/lib.rs && \
    echo "fn main() {}" > shardforge-config/src/lib.rs && \
    echo "fn main() {}" > shardforge-storage/src/lib.rs && \
    cargo build --release --bin shardforge --features sled --no-default-features && \
    rm -rf src bin shardforge-core/src shardforge-config/src shardforge-storage/src

COPY . .
RUN cargo build --release --bin shardforge --features sled --no-default-features

FROM gcr.io/distroless/cc-debian12

COPY --from=builder --chown=nonroot:nonroot /usr/src/shardforge/target/release/shardforge /shardforge
COPY --from=builder --chown=nonroot:nonroot /usr/src/shardforge/config/docker.toml /etc/shardforge/config.toml

USER nonroot:nonroot
WORKDIR /

EXPOSE 5432 9090

ENTRYPOINT ["/shardforge"]
CMD ["start", "--config", "/etc/shardforge/config.toml"]
