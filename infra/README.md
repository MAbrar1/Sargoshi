# infra
Container and deployment configuration for the gateway and its services.
Allowed here: `compose/` (docker-compose with Postgres, Redis, object store, LiveKit; from P03), `docker/`, `observability/`, `scripts/`.
Everything here must run locally and reproducibly from documented commands in docs/runbooks.
Not allowed: real secrets, keys, certificates or production hostnames.
