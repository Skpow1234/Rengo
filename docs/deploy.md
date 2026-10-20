# Ways to deploy Rengo

Short index of deployment options. Deep detail: [deployment.md](deployment.md) and [DOCKER.md](DOCKER.md).

> **Reality check:** `rengo start` now binds a **TCP JSON SQL** listener. Full gRPC tonic serve, Raft clustering, and K8s/Terraform remain ahead — see [milestones](milestones.md).

## 1. Local binary (development)

Best for hacking on the Rust workspace.

```bash
cargo build --release
./target/release/rengo init --single-node --data-dir ./data
./target/release/rengo start --config rengo.toml
```

- Config: [`rengo.toml`](../rengo.toml)
- Build help: [build.md](build.md), [getting-started.md](getting-started.md)

## 2. Docker (single container)

```bash
docker build -t rengo:local .
# or: make docker-build

docker run -d --name rengo-db \
  -v rengo-data:/data \
  -p 5432:5432 -p 9090:9090 \
  rengo:local start --config /etc/rengo/config.toml
```

Default ports: **5432** (node), **9090** (metrics / intended gRPC). See [endpoints.md](endpoints.md).

## 3. Docker Compose (profiles)

From the repo root:

| Profile | Command | Use case |
|---------|---------|----------|
| Default / production | `docker-compose up -d` | Single node |
| Dev | `docker-compose --profile dev up -d` | Hot reload / debug |
| Perf | `docker-compose --profile perf up -d` | Benchmarks |
| Cluster | `docker-compose --profile cluster up -d` | 3-node sketch |
| Monitoring | `docker-compose --profile monitoring up -d` | Prometheus + Grafana |

Details: [DOCKER.md](DOCKER.md).

## 4. Manual multi-node cluster (planned CLI)

Documented in [deployment.md](deployment.md):

```bash
rengo cluster bootstrap --node-id node-001 --bind 10.0.1.10:5432 ...
rengo cluster join --node-id node-002 --seed-nodes "10.0.1.10:5432" ...
```

Cluster join/bootstrap commands are still largely TODO in the CLI.

## 5. Infrastructure as Code (aspirational)

[deployment.md](deployment.md) sketches:

- **Terraform** AWS module layout
- Scripted cluster bring-up (`deploy-cluster.sh` style)
- Later: **Kubernetes** operator / Helm (Phase 4 in [roadmap.md](roadmap.md))

No committed Terraform/Helm charts ship in-tree yet; use Docker Compose for local multi-process experiments.

## 6. Monitoring sidecar

Compose `monitoring` profile + [`monitoring/prometheus.yml`](../monitoring/prometheus.yml). Wire metrics once the server exports them on `[monitoring].bind_address`.

## Recommended path today

1. **Develop:** local `cargo run` / release binary + `rengo.toml` (`storage.engine = "memory"` or `sled`).
2. **Package:** Docker / Compose production profile.
3. **Cluster / K8s / Terraform:** follow [deployment.md](deployment.md) as design intent; implement after the composition root and Raft transport land ([milestones.md](milestones.md)).
