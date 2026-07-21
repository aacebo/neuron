pub(crate) fn agent(alias: &str) -> String {
    format!(
        r#"
        jsonb_build_object(
            'status', {alias}.status,
            'description', {alias}.description,
            'skills', {alias}.skills,
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
            'name', {alias}.name,
            'display_name', {alias}.display_name
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
            'display_name', {alias}.display_name,
            'metadata', {alias}.metadata,
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
            'trace_id', {alias}.trace_id,
            'key', {alias}.key,
            'data', {alias}.data,
            'created_at', {alias}.created_at
        )
        "#
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    const ID: &str = "00000000-0000-4000-8000-000000000001";
    const RELATED_ID: &str = "00000000-0000-4000-8000-000000000002";
    const TRACE_ID: &str = "00000000-0000-4000-8000-000000000003";
    const CREATED_AT: &str = "2026-07-20T12:00:00Z";

    fn actor(agent: bool) -> Value {
        let mut actor = json!({
            "id": ID,
            "tenant_id": RELATED_ID,
            "external_id": "external-user",
            "role": if agent { "agent" } else { "user" },
            "name": "actor",
            "display_name": "Actor",
            "metadata": { "source": "test" },
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT
        });

        if agent {
            let object = actor.as_object_mut().expect("actor fixture must be an object");
            object.insert("status".into(), json!("online"));
            object.insert("description".into(), json!("Test agent"));
            object.insert("skills".into(), json!([]));
        }

        actor
    }

    fn actor_partial() -> Value {
        json!({
            "id": ID,
            "role": "user",
            "name": "actor",
            "display_name": "Actor"
        })
    }

    fn task_value() -> Value {
        json!({
            "id": ID,
            "trace_id": TRACE_ID,
            "name": "inference",
            "status": "queued",
            "input": null,
            "output": { "answer": 42 },
            "error": null,
            "attempts": 0,
            "max_attempts": 3,
            "started_at": null,
            "ended_at": null,
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT
        })
    }

    #[test]
    fn actor_projection_shape_supports_flattened_optional_agent() {
        let user: types::actors::Actor = serde_json::from_value(actor(false)).unwrap();
        assert!(user.agent.is_none());

        let agent: types::actors::Actor = serde_json::from_value(actor(true)).unwrap();
        let agent = agent.agent.expect("flattened agent should be populated");
        assert!(agent.status.is_online());
        assert!(agent.skills.is_empty());
    }

    #[test]
    fn chat_and_message_projection_shapes_deserialize() {
        let chat = json!({
            "id": RELATED_ID,
            "tenant_id": TRACE_ID,
            "name": "General",
            "created_by": actor_partial(),
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT,
            "closed_at": null
        });
        let chat: types::chats::Chat = serde_json::from_value(chat).unwrap();
        assert_eq!(chat.name.as_deref(), Some("General"));

        let message = json!({
            "id": ID,
            "chat": {
                "id": RELATED_ID,
                "tenant_id": TRACE_ID,
                "name": "General"
            },
            "content": [{ "type": "text", "text": "hello" }],
            "metadata": {},
            "created_by": actor_partial(),
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT
        });
        let message: types::chats::Message = serde_json::from_value(message).unwrap();
        assert_eq!(message.content[0].as_text(), Some("hello"));
    }

    #[test]
    fn resource_projection_shapes_deserialize() {
        let annotation = json!({
            "id": ID,
            "type": "entity",
            "label": "organization",
            "text": "OpenAI",
            "score": 0.99,
            "spans": [{ "start": 0, "end": 6 }],
            "created_at": CREATED_AT
        });
        let annotation: types::resources::Annotation = serde_json::from_value(annotation).unwrap();
        assert_eq!(annotation.spans[0].end, 6);

        let artifact = json!({
            "id": ID,
            "name": "embedding",
            "content": [{ "type": "json", "json": { "value": true } }],
            "embedding": [0.25, -0.5],
            "metadata": {},
            "created_by": actor(true),
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT
        });
        let artifact: types::resources::Artifact = serde_json::from_value(artifact).unwrap();
        assert_eq!(artifact.embedding, Some(vec![0.25, -0.5]));
    }

    #[test]
    fn task_event_skill_and_version_projection_shapes_deserialize() {
        let task: types::tasks::Task = serde_json::from_value(task_value()).unwrap();
        assert_eq!(task.status, types::tasks::TaskStatus::Queued);

        let event = json!({
            "id": RELATED_ID,
            "trace_id": TRACE_ID,
            "key": "task.created",
            "data": {
                "type": "task",
                "task": task_value()
            },
            "created_at": CREATED_AT
        });
        let event: types::events::Event = serde_json::from_value(event).unwrap();
        assert_eq!(event.key, "task.created");

        let skill = json!({
            "id": ID,
            "tenant_id": RELATED_ID,
            "name": "summarize",
            "display_name": "Summarize",
            "created_at": CREATED_AT
        });
        let skill: types::skills::Skill = serde_json::from_value(skill).unwrap();
        assert_eq!(skill.name, "summarize");

        let version = json!({
            "id": ID,
            "major": 1,
            "minor": 2,
            "patch": 3,
            "prerelease": null,
            "status": "published",
            "description": "Stable",
            "tags": ["text"],
            "input": null,
            "output": null,
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT
        });
        let version: types::skills::Version = serde_json::from_value(version).unwrap();
        assert_eq!((version.major, version.minor, version.patch), (1, 2, 3));
    }
}
