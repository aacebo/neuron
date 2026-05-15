CREATE TABLE IF NOT EXISTS message_artifacts (
    id          UUID                PRIMARY KEY NOT NULL,
    message_id  UUID                NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    type        TEXT                NOT NULL,
    content     JSONB               NOT NULL,
    embedding   VECTOR(384),
    created_at  TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_artifacts_type       ON message_artifacts(type);
CREATE INDEX IF NOT EXISTS idx_message_artifacts_embedding  ON message_artifacts USING hnsw(embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_message_artifacts_created_at ON message_artifacts(created_at);
