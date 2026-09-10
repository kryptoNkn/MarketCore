# Backend

The backend is a Cargo workspace. Each deployable service owns its handlers, application services, repositories, migrations, and runtime configuration. `common` contains only configuration shared by services; it must not contain business logic or database models.

## Local development

1. Copy `.env.example` to `.env` and replace `JWT_SECRET` with at least 32 random bytes.
2. Set `POSTGRES_PASSWORD` in the shell or in a local Compose environment file.
3. Start PostgreSQL:

```sh
docker compose up -d postgres
```

4. Run the auth service:

```sh
cargo run -p auth-service
```

The service applies its own migrations at startup. Liveness is available at `/health/live`, readiness at `/health/ready`.

## Auth API

- `POST /v1/auth/register` creates a user and returns a bearer access token.
- `POST /v1/auth/login` verifies credentials and returns a bearer access token.

Passwords are hashed with Argon2id. JWT signing keys and database credentials are supplied at runtime; production deployments must use a secret manager, managed PostgreSQL, TLS termination, and an external migration job. The included Compose file is for local development and does not replace a production orchestrator.

## Production boundary

Build the immutable service image with `docker build -f Dockerfile .`. Deploy the image behind an API gateway or ingress with resource limits, request timeouts, structured log collection, metrics, alerts, backups, and a rollback-capable migration process. Do not publish PostgreSQL outside the private network.
