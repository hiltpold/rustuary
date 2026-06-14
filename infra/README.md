# Infrastructure

Local placeholders:

- PostgreSQL for OLTP metadata and workflow state.
- MinIO as local S3-compatible object storage.
- NATS as a lightweight queue placeholder.

Run:

```bash
docker compose -f infra/docker-compose.yml up
```

These choices are placeholders, not final architecture commitments.
