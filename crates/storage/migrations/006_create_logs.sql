CREATE TABLE IF NOT EXISTS logs (
    id              UUID            PRIMARY KEY NOT NULL,
    trace_id        UUID            NOT NULL,
    level           TEXT            NOT NULL,
    source          TEXT            NOT NULL,
    message         TEXT            NOT NULL,
    context         JSONB,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_logs_trace        ON logs(trace_id, level, source);
CREATE INDEX IF NOT EXISTS idx_logs_level_source ON logs(level, source);
CREATE INDEX IF NOT EXISTS idx_logs_created_at   ON logs(trace_id, created_at DESC);
