# Endpoints & Network APIs

Rengo exposes:

1. A **TCP JSON SQL** listener on `[node].bind_address` (live via `DatabaseNode::start` / `rengo start`)
2. A **gRPC** service defined in [`proto/database.proto`](../proto/database.proto) (stubs until `grpc-enabled` + protoc)

Default bind addresses come from [`rengo.toml`](../rengo.toml). See [milestones](milestones.md).

## Default ports

| Port | Config key | Purpose |
|------|------------|---------|
| `5432` | `[node].bind_address` | TCP JSON SQL (and future PG/gRPC client bind) |
| `9090` | `[monitoring].bind_address` | Metrics / monitoring bind |

## TCP JSON SQL protocol (live)

Newline-delimited JSON over TCP.

**Request**

```json
{"sql":"CREATE TABLE t (id INTEGER, name VARCHAR(64))"}
```

**Response**

```json
{"ok":true,"columns":[],"rows":[],"affected_rows":1,"error":null}
```

Example with `nc` / PowerShell after `rengo start`:

```bash
echo '{"sql":"SELECT id, name FROM t"}' | nc 127.0.0.1 5432
```

CLI without a long-running server (in-process node):

```bash
rengo sql "SELECT 1" --config rengo.toml
```


Docker Compose maps the same ports (and variants for multi-profile setups):

| Profile | Client port | gRPC / metrics port |
|---------|-------------|---------------------|
| Production / default | `5432` | `9090` |
| Development | `5433` → `5432` | `9091` → `9090` |
| Performance | `5434` → `5432` | `9092` → `9090` |
| Cluster node 2 | `5435` | `9093` |
| Cluster node 3 | `5436` | `9094` |

Full Docker port tables: [DOCKER.md](DOCKER.md).

## gRPC service: `rengo.rpc.DatabaseService`

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

Until the server fully listens, most workflows go through the `rengo` CLI:

```bash
rengo init --single-node --data-dir ./data
rengo start --config rengo.toml
rengo status --detailed
rengo sql --query "SELECT 1"
```

Cluster-oriented commands (bootstrap / join / status) are sketched in the CLI crate; see [deployment](deployment.md) and [CLI summary](cli-implementation-summary.md).

## Related docs

- [Getting started](getting-started.md)
- [Deployment](deployment.md)
- [Docker](DOCKER.md)
- [Architecture](architecture.md)
