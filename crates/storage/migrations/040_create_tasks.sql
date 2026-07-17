CREATE TABLE IF NOT EXISTS tasks (
    id              UUID            PRIMARY KEY,
    parent_id       UUID            REFERENCES tasks(id) ON DELETE CASCADE,
    chat_id         UUID            NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    message_id      UUID            REFERENCES messages(id) ON DELETE CASCADE,
    agent_id        UUID            REFERENCES agents(id) ON DELETE CASCADE,
    name            TEXT            NOT NULL,
    status          TEXT            NOT NULL,
    input           JSONB,
    output          JSONB,
    error           JSONB,
    attempts        INT             NOT NULL DEFAULT 0,
    max_attempts    INT             NOT NULL DEFAULT 3,
    started_at      TIMESTAMPTZ,
    ended_at        TIMESTAMPTZ,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
