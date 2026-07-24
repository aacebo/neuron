pub(crate) fn agent(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'status', {alias}.status,
            'description', {alias}.description,
            'secret', {alias}.secret,
            'instances', {alias}.instances,
            'skills', {alias}.skills
        )
        "#
    )
}

pub(crate) fn actor_partial(alias: &str) -> String {
    let agent = agent("agent");
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'role', {alias}.role,
            'name', {alias}.name
        ) || COALESCE((
            SELECT {agent}
            FROM agents agent
            WHERE agent.actor_id = {alias}.id
        ), '{{}}'::jsonb)
        "#
    )
}

pub(crate) fn actor(alias: &str) -> String {
    let agent = agent("agent");
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'tenant_id', {alias}.tenant_id,
            'external_id', {alias}.external_id,
            'role', {alias}.role,
            'name', {alias}.name,
            'metadata', {alias}.metadata,
            'embedding', CASE
                WHEN {alias}.embedding IS NULL THEN NULL
                ELSE ({alias}.embedding::text)::jsonb
            END,
            'created_at', {alias}.created_at,
            'updated_at', {alias}.updated_at
        ) || COALESCE((
            SELECT {agent}
            FROM agents agent
            WHERE agent.actor_id = {alias}.id
        ), '{{}}'::jsonb)
        "#
    )
}

pub(crate) fn chat_partial(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'tenant_id', {alias}.tenant_id,
            'name', {alias}.name
        )
        "#
    )
}

pub(crate) fn chat(alias: &str) -> String {
    let creator = actor_partial("creator");
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'tenant_id', {alias}.tenant_id,
            'name', {alias}.name,
            'created_by', (
                SELECT {creator}
                FROM actors creator
                WHERE creator.id = {alias}.created_by_id
            ),
            'created_at', {alias}.created_at,
            'updated_at', {alias}.updated_at,
            'closed_at', {alias}.closed_at
        )
        "#
    )
}

pub(crate) fn message(alias: &str) -> String {
    let chat = chat_partial("chat");
    let creator = actor_partial("creator");
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'chat', (
                SELECT {chat}
                FROM chats chat
                WHERE chat.id = {alias}.chat_id
            ),
            'content', {alias}.content,
            'metadata', {alias}.metadata,
            'embedding', CASE
                WHEN {alias}.embedding IS NULL THEN NULL
                ELSE ({alias}.embedding::text)::jsonb
            END,
            'created_by', (
                SELECT {creator}
                FROM actors creator
                WHERE creator.id = {alias}.created_by_id
            ),
            'created_at', {alias}.created_at,
            'updated_at', {alias}.updated_at
        )
        "#
    )
}

pub(crate) fn annotation(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'type', {alias}.type,
            'label', {alias}.label,
            'text', {alias}.text,
            'score', {alias}.score,
            'spans', {alias}.spans,
            'created_at', {alias}.created_at
        )
        "#
    )
}

pub(crate) fn artifact(alias: &str) -> String {
    let creator = actor("creator");
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'name', {alias}.name,
            'content', {alias}.content,
            'embedding', CASE
                WHEN {alias}.embedding IS NULL THEN NULL
                ELSE ({alias}.embedding::text)::jsonb
            END,
            'metadata', {alias}.metadata,
            'created_by', (
                SELECT {creator}
                FROM actors creator
                WHERE creator.id = {alias}.created_by_id
            ),
            'created_at', {alias}.created_at,
            'updated_at', {alias}.updated_at
        )
        "#
    )
}

pub(crate) fn task(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'trace_id', {alias}.trace_id,
            'parent_id', {alias}.parent_id,
            'name', {alias}.name,
            'status', {alias}.status,
            'input', {alias}.input,
            'output', {alias}.output,
            'error', {alias}.error,
            'attempts', {alias}.attempts,
            'max_attempts', {alias}.max_attempts,
            'started_at', {alias}.started_at,
            'ended_at', {alias}.ended_at,
            'created_at', {alias}.created_at,
            'updated_at', {alias}.updated_at
        )
        "#
    )
}

pub(crate) fn event(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'tenant_id', {alias}.tenant_id,
            'trace_id', {alias}.trace_id,
            'key', {alias}.key,
            'data', {alias}.data,
            'created_at', {alias}.created_at
        )
        "#
    )
}

pub(crate) fn log(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'id', {alias}.id,
            'trace_id', {alias}.trace_id,
            'tenant_id', {alias}.tenant_id,
            'task_id', {alias}.task_id,
            'level', {alias}.level,
            'source', {alias}.source,
            'message', {alias}.message,
            'fields', {alias}.fields,
            'created_by_id', {alias}.created_by_id,
            'created_at', {alias}.created_at
        )
        "#
    )
}
