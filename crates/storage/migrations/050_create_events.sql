CREATE TABLE IF NOT EXISTS events (
    id              UUID            PRIMARY KEY,
    trace_id        UUID            NOT NULL,
    tenant_id       UUID            NOT NULL,
    actor_id        UUID            REFERENCES actors(id) ON DELETE CASCADE,
    chat_id         UUID            REFERENCES chats(id) ON DELETE CASCADE,
    message_id      UUID            REFERENCES messages(id) ON DELETE CASCADE,
    task_id         UUID            REFERENCES tasks(id) ON DELETE CASCADE,
    key             TEXT            NOT NULL,
    data            JSONB           NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
