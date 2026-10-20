# Milestones & Project Status

This page summarizes **roadmap phases**, **what the code actually delivers today**, and how those relate. The long-form plan lives in [roadmap.md](roadmap.md).

## Honest status (current)

ShardForge is an **early foundation**: crates and modules for storage, SQL, Raft, config, and CLI exist, but they are **not yet composed into a running database node**.

| Area | State |
|------|--------|
| Workspace crates (`core`, `config`, `storage`) | Usable |
| In-memory / Sled / RocksDB KV engines | Partial (feature-gated) |
| SQL parser / planner / executor | Partial (library-level) |
| Transaction / MVCC modules | Partial; not fully wired to SQL |
| Raft log / vote / append handlers | Partial; election start still TODO |
| gRPC / network server | Stub (simulated start, placeholder RPCs) |
| CLI `init` / `start` / `sql` | Commands present; server start is largely a placeholder |
| Sharding / multi-region / enterprise security | Design / docs only |

CLI and Docker packaging are ahead of a fully listening server. Prefer this page over marketing checklists in older docs when prioritizing work.

## Roadmap phases (planned)

From [roadmap.md](roadmap.md) — **36-month** plan:

### Phase 1 — Foundation (months 1–8)

Single-node database: storage, SQL, transactions, CLI, Docker.

**Target deliverables:** single-node SQL DB, CLI, container images  
**Success criteria (aspirational):** TPC-C > 10k tpmC, broad PostgreSQL syntax

### Phase 2 — Distributed core (months 9–18)

Raft, hashing/sharding, distributed transactions, distributed query planning.

**Target deliverables:** multi-node cluster with consensus + sharding  
**Success criteria (aspirational):** scale to ~16 nodes, distributed TPC-C > 50k tpmC

### Phase 3 — Production readiness (months 19–28)

Advanced SQL, observability, TLS/RBAC, backup/DR, compliance.

**Target deliverables:** enterprise ops features  
**Success criteria (aspirational):** high availability targets, SOC 2–oriented controls

### Phase 4 — Scale & innovation (months 29–36)

Geo distribution, analytics, K8s operator, multi-cloud, extensions.

**Target deliverables:** global / ecosystem capabilities  
**Success criteria (aspirational):** 100+ node / large-scale deployments

## Milestone checklist (engineering view)

Use this as a practical board. Items marked done mean **meaningful code exists**, not that the product is production-ready.

### Near-term (make it a real single node)

- [x] Project workspace and core types
- [x] Config load + validation
- [x] Storage trait + memory engine
- [x] SQL parse / plan / execute building blocks
- [x] Raft module structure and RPC handlers (local)
- [ ] **Composition root** (`DatabaseNode` / server lifecycle wiring SQL + storage + network)
- [ ] Real gRPC listen + proto codegen path
- [ ] Durable catalog + SQL path through transactions
- [ ] CLI `start` actually boots the server
- [ ] Integration tests discovered and green in CI

### Mid-term (distributed)

- [ ] Raft leader election + peer transport
- [ ] Log apply to state machine / storage
- [ ] Cluster bootstrap / join via CLI
- [ ] Hash sharding + metadata routing
- [ ] Cross-shard / 2PC transactions

### Later

- [ ] TLS, RBAC, audit
- [ ] Backup / PITR
- [ ] K8s operator / Helm
- [ ] Multi-region and advanced analytics

## Success metrics by phase (roadmap targets)

| Phase | Users (target) | Performance (target) | Availability (target) | Features |
|-------|----------------|----------------------|------------------------|----------|
| 1 | 10 | 10k tpmC | 99.9% | Single-node SQL |
| 2 | 100 | 50k tpmC | 99.95% | Distributed core |
| 3 | 1,000 | 200k tpmC | 99.99% | Enterprise ready |
| 4 | 10,000 | 1M tpmC | 99.999% | Global scale |

These are **goals**, not measured results. Current benches are mostly in-memory storage microbenchmarks — see [performance.md](performance.md).

## Related docs

- [roadmap.md](roadmap.md) — full phased plan
- [success-metrics.md](success-metrics.md) — KPIs
- [risk-assessment.md](risk-assessment.md) — risks
- [getting-started.md](getting-started.md) — run locally
- [endpoints.md](endpoints.md) — APIs / ports
