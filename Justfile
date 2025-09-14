start-db:
    podman start tremolo-db || podman run \
        --name tremolo-db \
        --rm \
        -p 5432:5432 \
        -e POSTGRES_PASSWORD=password \
        -e POSTGRES_DB=tremolo-db \
        -d docker.io/library/postgres:18rc1

    sleep 0.5
    sqlx migrate run --source src/server/migrations

stop-db:
    podman stop tremolo-db || echo "Postgres is not running"

reset-db: stop-db start-db

serve: reset-db
    cargo run -- server

check: reset-db
    cargo clippy

psql:
    psql $DATABASE_URL
