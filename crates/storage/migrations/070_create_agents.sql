CREATE TABLE IF NOT EXISTS agents (
    actor_id        UUID        PRIMARY KEY REFERENCES actors(id) ON DELETE CASCADE,
    status          TEXT        NOT NULL, -- online, offline
    description     TEXT        NOT NULL,
);

CREATE TABLE IF NOT EXISTS agent_skills (
    agent_id            UUID        NOT NULL REFERENCES agents(actor_id) ON DELETE CASCADE,
    skill_version_id    UUID        NOT NULL REFERENCES skill_versions(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (agent_id, skill_version_id)
);
