# Worker service

Placeholder for background jobs and workflow execution.

Possible responsibilities:

- Fetch run metadata from API/PostgreSQL.
- Download normalized input artifacts from object storage.
- Invoke the Rust engine.
- Write Arrow/Parquet result artifacts.
- Update job state and append audit events.

Do not put user-facing API authorization rules here; those belong in the API service.
