# ShardForge

Distributed database in Rust — ACID transactions, SQL, Raft consensus, and sharding. CLI-first, cloud-native packaging.

[![Rust](https://img.shields.io/badge/Rust-1.80+-000000.svg?style=flat-square&logo=rust)](https://rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-async-000000.svg?style=flat-square&logo=tokio)](https://tokio.rs/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](Cargo.toml)

> **Phase:** early foundation. Storage, SQL, Raft, and CLI modules exist; a fully wired listening database node is still in progress. See [docs/milestones.md](docs/milestones.md).

## Quick start

```bash
cargo build --release
./target/release/shardforge init --single-node --data-dir ./data
./target/release/shardforge start --config shardforge.toml
./target/release/shardforge status --detailed
```

Docker / Compose and platform build notes: **[Getting started](docs/getting-started.md)**.

## Documentation

| Topic | Link |
|-------|------|
| Docs home | [docs/README.md](docs/README.md) |
| Getting started | [docs/getting-started.md](docs/getting-started.md) |
| Endpoints (ports & gRPC) | [docs/endpoints.md](docs/endpoints.md) |
| Milestones & status | [docs/milestones.md](docs/milestones.md) |
| Ways to deploy | [docs/deploy.md](docs/deploy.md) |
| Docker profiles | [docs/DOCKER.md](docs/DOCKER.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |
| Roadmap | [docs/roadmap.md](docs/roadmap.md) |

## What's implemented

- Workspace crates: `shardforge-core`, `shardforge-config`, `shardforge-storage`
- Pluggable storage (memory / sled / rocksdb)
- SQL, transaction, Raft, and network modules (partial)
- CLI, config, Docker, and CI scaffolding

Full checklist: [docs/milestones.md](docs/milestones.md).

## Key ideas

- **Rust** — memory safety and performance for the data plane
- **CLI-first** — `init`, `start`, `status`, `sql` from day one
- **Open source** — MIT OR Apache-2.0
- **Cloud-native packaging** — Docker Compose profiles and deploy docs

## License

MIT OR Apache-2.0 — see `Cargo.toml`.
