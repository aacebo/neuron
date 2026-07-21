CREATE TABLE IF NOT EXISTS agents (
    actor_id        UUID        PRIMARY KEY REFERENCES actors(id) ON DELETE CASCADE,
    status          TEXT        NOT NULL, -- online, offline
    description     TEXT        NOT NULL,
    skills          JSONB       NOT NULL DEFAULT '[]'
);
