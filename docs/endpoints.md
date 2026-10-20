# Endpoints & Network APIs

ShardForge exposes a **gRPC** service defined in [`proto/database.proto`](../proto/database.proto). Default bind addresses come from [`shardforge.toml`](../shardforge.toml).

> **Status:** The protocol and stubs exist. The gRPC server currently **simulates** a listen loop and returns placeholder responses until protoc generation and service wiring are finished. See [milestones](milestones.md).

## Default ports

| Port | Config key | Purpose |
|------|------------|---------|
| `5432` | `[node].bind_address` | Primary node / client bind (intended SQL / wire access) |
| `9090` | `[monitoring].bind_address` | Metrics / monitoring bind |

Docker Compose maps the same ports (and variants for multi-profile setups):

| Profile | Client port | gRPC / metrics port |
|---------|-------------|---------------------|
| Production / default | `5432` | `9090` |
| Development | `5433` → `5432` | `9091` → `9090` |
| Performance | `5434` → `5432` | `9092` → `9090` |
| Cluster node 2 | `5435` | `9093` |
| Cluster node 3 | `5436` | `9094` |

Full Docker port tables: [DOCKER.md](DOCKER.md).

## gRPC service: `shardforge.rpc.DatabaseService`

### Cluster management

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `JoinCluster` | `JoinClusterRequest` | stream `JoinClusterResponse` | Join a cluster (accepted / rejected / topology updates) |
| `LeaveCluster` | `LeaveClusterRequest` | `LeaveClusterResponse` | Leave a cluster (graceful or forced) |
| `Heartbeat` | `HeartbeatRequest` | `HeartbeatResponse` | Liveness + optional cluster update |
| `GetClusterTopology` | `TopologyRequest` | `TopologyResponse` | Current topology and optional node stats |

### Data / SQL operations

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `ExecuteQuery` | `QueryRequest` | stream `QueryResponse` | Run SQL (result set, error, or progress) |
| `ExecuteBatch` | `BatchRequest` | `BatchResponse` | Run multiple queries as a batch |
| `StreamQuery` | `StreamQueryRequest` | stream `StreamQueryResponse` | Streaming query with batch size control |

### Transactions

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `BeginTransaction` | `TransactionRequest` | `TransactionResponse` | Start a transaction (isolation, timeout, read-only) |
| `PrepareTransaction` | `PrepareRequest` | `PrepareResponse` | 2PC prepare |
| `CommitTransaction` | `CommitRequest` | `CommitResponse` | Commit |
| `RollbackTransaction` | `RollbackRequest` | `RollbackResponse` | Rollback |

### Administration

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `GetClusterStatus` | `StatusRequest` | `StatusResponse` | Health, nodes, optional metrics |
| `GetMetrics` | `MetricsRequest` | stream `MetricsResponse` | Time-range / named metrics |
| `ExecuteAdminCommand` | `AdminCommandRequest` | `AdminCommandResponse` | Admin CLI-style command |

### Metadata

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `GetSchema` | `SchemaRequest` | `SchemaResponse` | Schema and tables |
| `UpdateSchema` | `UpdateSchemaRequest` | `UpdateSchemaResponse` | Schema mutations |
| `GetShardMap` | `ShardMapRequest` | `ShardMapResponse` | Shard placement for a table |

## CLI surface (local)

Until the server fully listens, most workflows go through the `shardforge` CLI:

```bash
shardforge init --single-node --data-dir ./data
shardforge start --config shardforge.toml
shardforge status --detailed
shardforge sql --query "SELECT 1"
```

Cluster-oriented commands (bootstrap / join / status) are sketched in the CLI crate; see [deployment](deployment.md) and [CLI summary](cli-implementation-summary.md).

## Related docs

- [Getting started](getting-started.md)
- [Deployment](deployment.md)
- [Docker](DOCKER.md)
- [Architecture](architecture.md)
