CREATE TABLE agents (
       id           UUID      PRIMARY KEY DEFAULT (uuidv7()),
       name         TEXT      NOT NULL UNIQUE,
       is_connected BOOLEAN   NOT NULL DEFAULT TRUE,
       last_seen    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE agents_tokens (
       token     TEXT      PRIMARY KEY DEFAULT (substr(md5(random()::text), 1, 32)),
       Last_used TIMESTAMP
);

CREATE TABLE apps (
       id          UUID PRIMARY KEY DEFAULT (uuidv7()),
       name        TEXT NOT NULL UNIQUE,
       description TEXT,

       created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
       updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE deployments (
       id         UUID      PRIMARY KEY DEFAULT (uuidv7()),
       hash       TEXT      NOT NULL,
       start_time TIMESTAMP NOT NULL,
       end_time   TIMESTAMP,

       app_id UUID NOT NULL REFERENCES apps(id)
);

CREATE TABLE deployments_logs (
       id        BIGINT    PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
       timestamp TIMESTAMP NOT NULL,
       message   TEXT      NOT NULL,

       deployment_id UUID REFERENCES deployments(id)
);
