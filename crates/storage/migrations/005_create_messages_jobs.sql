CREATE TABLE IF NOT EXISTS messages_jobs (
    message_id  UUID            NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    job_id      UUID            NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    PRIMARY KEY(message_id, job_id)
);
