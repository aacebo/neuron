CREATE TABLE IF NOT EXISTS jobs (
    id              UUID        PRIMARY KEY NOT NULL,
    name            TEXT        NOT NULL,
    status          TEXT        NOT NULL,
    error           JSONB,
    attempts        INT NOT NULL DEFAULT 0,
    max_attempts    INT NOT NULL DEFAULT 3,
    started_at      TIMESTAMPTZ,
    ended_at        TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
