# CI/CD

## Workflows

| Workflow | Trigger | Role |
|----------|---------|------|
| [`ci.yml`](../.github/workflows/ci.yml) | push / PR to `main`, `develop` | **Required gate** — fmt, clippy (`-D warnings`), matrix tests, audit, deny, docs, Docker smoke |
| [`nightly.yml`](../.github/workflows/nightly.yml) | daily + manual | machete, deny, miri, benches |
| [`dependency-updates.yml`](../.github/workflows/dependency-updates.yml) | weekly | `cargo update` + tests + audit → PR |
| [`release.yml`](../.github/workflows/release.yml) | version tags `v*.*.*` | multi-platform release binaries / Docker |

Redundant workflows (`comprehensive`, `pr-checks`, `warnings`, `code-quality`, `project-health`) were removed in favor of the above.

## Guarantees

- **No soft-fails** on the main gate (`|| echo continuing` removed)
- **Concurrency** cancels stale runs on the same ref
- **`ci-success`** job aggregates required results for a single branch-protection check
- Clippy **denies `clippy::correctness`** (real bugs); style/docs debt does not soft-fail the gate via `|| true` — it is simply out of the deny set until cleaned up
- Docker image ships with `/etc/shardforge/config.toml` and starts the TCP SQL listener by default

## Local equivalents

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features sled --no-default-features -- -D clippy::correctness -A missing_docs
cargo test --workspace --features sled --no-default-features
cargo audit
```
