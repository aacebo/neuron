CREATE TABLE IF NOT EXISTS artifacts (
    id              UUID                PRIMARY KEY,
    chat_id         UUID                NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    message_id      UUID                REFERENCES messages(id) ON DELETE CASCADE,
    task_id         UUID                REFERENCES tasks(id) ON DELETE CASCADE,
    name            TEXT                NOT NULL,
    content         JSONB               NOT NULL DEFAULT '[]',
    embedding       VECTOR(384),
    metadata        JSONB               NOT NULL DEFAULT '{}',
    created_by_id   UUID                REFERENCES actors(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_artifacts_name       ON artifacts(name);
CREATE INDEX IF NOT EXISTS idx_artifacts_embedding  ON artifacts USING hnsw(embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_artifacts_created_at ON artifacts(created_at);
