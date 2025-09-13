recreate-db:
    sqlx database reset -y --source src/server/migrations/
    sqlite3 dev.db .schema > schema.sql
