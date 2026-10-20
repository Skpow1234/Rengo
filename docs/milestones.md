# Milestones & Project Status

This page summarizes **roadmap phases**, **what the code delivers today**, and near-term priorities.  
The exhaustive backlog (**500+ items**) lives in [milestones-backlog.md](milestones-backlog.md). The long-form plan is in [roadmap.md](roadmap.md).

## Honest status (current)

ShardForge now has a **single-node composition root** (`DatabaseNode`) that opens storage, loads a durable catalog, runs SQL through the executor + transaction manager, and can listen for TCP JSON SQL. gRPC remains stubbed without protoc; Raft election/transport and sharding are still ahead.

| Area | State |
|------|--------|
| Workspace crates (`core`, `config`, `storage`) | Usable |
| `DatabaseNode` composition root | **Implemented** |
| TCP JSON SQL listener (`shardforge start`) | **Implemented** |
| CLI `sql` / `status` against node | **Implemented** |
| Durable catalog + `scan_prefix` | **Implemented** |
| In-memory / Sled / RocksDB KV engines | Partial (feature-gated) |
| SQL parser / planner / executor | Partial |
| Transaction manager lifecycle around SQL | Partial (auto begin/commit; writes still mostly direct to KV) |
| Raft log / vote / append handlers | Partial; election start still TODO |
| gRPC tonic serve | Stub (service can attach a node; no real serve yet) |
| Sharding / multi-region / enterprise security | Design / docs only |

## What just landed

- [x] `DatabaseNode` owns config → storage → catalog → SQL → transactions
- [x] `shardforge start` binds and serves newline-delimited JSON `{"sql":"..."}`
- [x] `shardforge sql "..."` executes in-process via `DatabaseNode`
- [x] Catalog persisted as JSON; row ids via `__seq__*` ; table scans via `scan_prefix`
- [x] Integration test discovery via `[[test]]` + `tests/node_integration.rs`
- [x] **Robust CI** — single fail-hard pipeline; removed soft-fail duplicate workflows
- [x] **Session transactions** — `BEGIN` / `COMMIT` / `ROLLBACK` tracked on the node
- [x] **Sled restart durability** — catalog + rows survive reopen (`test_sled_catalog_and_data_survive_restart`)

## Near-term next (still open)

1. Real tonic gRPC serve (`grpc-enabled` + protoc) streaming query results
2. Wire SQL writes through MVCC write-sets (not only txn lifecycle)
3. Raft election + peer transport, then propose writes through consensus
4. Unify dual CLI (`bin/` vs `shardforge-cli`) and workspace membership
5. RocksDB restart durability parity with sled
6. Session-local write buffering so ROLLBACK undoes DML (today txn flag is real; KV writes still land immediately)

Full list: [milestones-backlog.md](milestones-backlog.md).

## Roadmap phases (planned)

### Phase 1 — Foundation (months 1–8)

Single-node database: storage, SQL, transactions, CLI, Docker.

### Phase 2 — Distributed core (months 9–18)

Raft, hashing/sharding, distributed transactions, distributed query planning.

### Phase 3 — Production readiness (months 19–28)

Advanced SQL, observability, TLS/RBAC, backup/DR, compliance.

### Phase 4 — Scale & innovation (months 29–36)

Geo distribution, analytics, K8s operator, multi-cloud, extensions.

## Related docs

- [milestones-backlog.md](milestones-backlog.md) — **500+** engineering milestones
- [roadmap.md](roadmap.md) — phased plan
- [getting-started.md](getting-started.md) — run locally
- [endpoints.md](endpoints.md) — ports / gRPC / TCP SQL
- [deploy.md](deploy.md) — deploy options
