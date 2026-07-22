CREATE TABLE IF NOT EXISTS annotations (
    id          UUID                PRIMARY KEY,
    message_id  UUID                NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    task_id     UUID                REFERENCES tasks(id) ON DELETE CASCADE,
    type        TEXT                NOT NULL,
    label       TEXT                NOT NULL,
    text        TEXT                NOT NULL,
    score       DOUBLE PRECISION    NOT NULL,
    spans       JSONB               NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_annotations_type       ON annotations(type);
CREATE INDEX IF NOT EXISTS idx_annotations_label      ON annotations(label);
CREATE INDEX IF NOT EXISTS idx_annotations_created_at ON annotations(created_at);
