CREATE TABLE IF NOT EXISTS skills (
    id              UUID        PRIMARY KEY,
    tenant_id       UUID        NOT NULL,
    name            TEXT        UNIQUE NOT NULL,
    display_name    TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS skill_versions (
    id              UUID        PRIMARY KEY,
    skill_id        UUID        NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    major           INT         NOT NULL,
    minor           INT         NOT NULL,
    patch           INT         NOT NULL,
    prerelease      TEXT,
    status          TEXT        NOT NULL, -- draft, published, deprecated
    description     TEXT        NOT NULL,
    tags            TEXT[]      NOT NULL,
    input           JSONB,
    output          JSONB,
    embedding       VECTOR(384),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE NULLS NOT DISTINCT (
        skill_id,
        major,
        minor,
        patch,
        prerelease
    )
);

CREATE INDEX IF NOT EXISTS idx_skill_versions_skill_status
ON skill_versions (skill_id, status);

CREATE INDEX IF NOT EXISTS idx_skill_versions_published
ON skill_versions (skill_id, major DESC, minor DESC, patch DESC)
WHERE status = 'published';
