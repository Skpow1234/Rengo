# Getting Started

Quick ways to build and run ShardForge locally. For production layouts, see [deployment](deployment.md) and [Docker](DOCKER.md).

> **Note:** Single-node `DatabaseNode` can open storage, run SQL, and listen for TCP JSON SQL via `shardforge start`. Full gRPC tonic serve, Raft clustering, and durable MVCC are still in progress. See [milestones](milestones.md).

## Prerequisites

- [Rust 1.80+](https://rustup.rs/)
- On Linux: `build-essential`, `pkg-config`, `libssl-dev`, `clang`
- Optional: [Docker](https://docker.com/get-started) for container workflows

Storage engines:

| Engine | Feature | Notes |
|--------|---------|--------|
| Sled (default) | `sled` | Pure Rust; best for Windows / local dev |
| RocksDB | `rocksdb` | Production-oriented; needs C++ toolchain |
| Memory | always | In-process; tests and demos |

More build detail: [build.md](build.md).

## Build from source

```bash
git clone https://github.com/Skpow1234/ShardForge.git
cd ShardForge

# Linux system deps (example)
sudo apt-get install -y build-essential pkg-config libssl-dev clang

cargo build --release

./target/release/shardforge init --single-node --data-dir ./data
./target/release/shardforge start --config shardforge.toml
./target/release/shardforge status --detailed
./target/release/shardforge sql "SELECT 1 as welcome"
```

Windows (PowerShell, sled recommended):

```powershell
.\scripts\build.ps1 -Test
.\target\release\shardforge.exe init --single-node --data-dir .\data
```

## Docker (image run)

```bash
mkdir -p data

docker pull shardforge/shardforge:latest   # when published

docker run --rm -v $(pwd)/data:/data shardforge/shardforge:latest \
  init --single-node --data-dir /data

docker run -d --name shardforge-db \
  -v $(pwd)/data:/data \
  -p 5432:5432 \
  shardforge/shardforge:latest \
  start --config /data/shardforge.toml

docker run --rm --network container:shardforge-db \
  shardforge/shardforge:latest status

docker stop shardforge-db && docker rm shardforge-db
```

## Docker Compose (local)

```bash
# Default / production profile
docker-compose up -d

docker-compose exec shardforge init --single-node --data-dir /data/shardforge
docker-compose exec shardforge status --detailed
docker-compose logs -f shardforge
docker-compose down
```

Other profiles (`dev`, `perf`, `cluster`, `monitoring`): [DOCKER.md](DOCKER.md).

## Configuration

Default config: [`shardforge.toml`](../shardforge.toml)

Key settings:

- `[node].bind_address` — client / node listen (default `127.0.0.1:5432`)
- `[storage].engine` — `memory` | `sled` | `rocksdb`
- `[monitoring].bind_address` — metrics (default `127.0.0.1:9090`)

## Next steps

- [Endpoints](endpoints.md) — gRPC RPCs and ports
- [Milestones](milestones.md) — what works today vs roadmap
- [Deployment](deployment.md) — single-node, cluster, IaC sketches
- [Architecture](architecture.md) — system design
