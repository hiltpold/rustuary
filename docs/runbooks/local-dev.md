# Local development runbook

## Prerequisites

- Rust toolchain
- Go
- Node.js and pnpm
- Python 3.10+
- Docker, if using local infrastructure

## Useful commands

```bash
./scripts/check.sh
cargo test -p rustuary-core
(cd services/api && go test ./...)
docker compose -f infra/docker-compose.yml up
```

## Local infrastructure

The Docker Compose file starts PostgreSQL, MinIO, and NATS placeholders. These are not required for the Rust core.
