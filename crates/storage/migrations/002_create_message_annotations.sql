CREATE TABLE IF NOT EXISTS message_annotations (
    id          UUID                PRIMARY KEY NOT NULL,
    message_id  UUID                NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    type        TEXT                NOT NULL,
    label       TEXT                NOT NULL,
    text        TEXT                NOT NULL,
    score       DOUBLE PRECISION    NOT NULL,
    spans       JSONB               NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_annotations_type       ON message_annotations(type);
CREATE INDEX IF NOT EXISTS idx_message_annotations_label      ON message_annotations(label);
CREATE INDEX IF NOT EXISTS idx_message_annotations_created_at ON message_annotations(created_at);
