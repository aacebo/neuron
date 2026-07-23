CREATE INDEX IF NOT EXISTS idx_events_tenant_cursor
ON events (tenant_id, created_at, id);
