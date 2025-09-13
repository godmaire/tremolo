CREATE TABLE agents (
       id           INTEGER  PRIMARY KEY NOT NULL,
       name         TEXT     NOT NULL UNIQUE,
       is_connected BOOLEAN  NOT NULL DEFAULT TRUE,
       last_seen    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE apps (
       id          INTEGER PRIMARY KEY NOT NULL,
       name        TEXT    NOT NULL UNIQUE,
       description TEXT,

       created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
       updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE deployments (
       id         INTEGER  PRIMARY KEY NOT NULL,
       hash       TEXT     NOT NULL,
       start_time DATETIME NOT NULL,
       end_time   DATETIME,

       app_id INTEGER NOT NULL REFERENCES apps(id)
);

CREATE TABLE deployments_logs (
       id        INTEGER  PRIMARY KEY NOT NULL,
       timestamp DATETIME NOT NULL,
       message   TEXT     NOT NULL,

       deployment_id INTEGER REFERENCES deployments(id)
);
