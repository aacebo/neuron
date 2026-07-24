CREATE TABLE IF NOT EXISTS logs (
    id              UUID        PRIMARY KEY,
    trace_id        UUID        NOT NULL,
    tenant_id       UUID        NOT NULL,
    task_id         UUID        REFERENCES tasks(id) ON DELETE CASCADE,
    level           TEXT        NOT NULL,
    source          TEXT        NOT NULL,
    message         TEXT        NOT NULL,
    fields          JSONB       NOT NULL DEFAULT '{}',
    created_by_id   UUID        REFERENCES actors(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_logs_trace_created
ON logs (trace_id, created_at);

CREATE INDEX idx_logs_task_created
ON logs (task_id, created_at);

CREATE INDEX idx_logs_tenant_level_created
ON logs (tenant_id, level, created_at DESC);

CREATE INDEX idx_logs_created_at_brin
ON logs USING BRIN (created_at);
