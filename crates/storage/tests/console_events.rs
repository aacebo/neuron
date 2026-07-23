use sqlx::PgPool;
use storage::{ActorStorage, ChatStorage, EventCursor, EventStorage};
use types::actors::{Actor, Agent, AgentStatus, Role};
use types::chats::ChatMembers;

fn agent(tenant_id: uuid::Uuid) -> Actor {
    let now = chrono::Utc::now();
    Actor {
        id: uuid::Uuid::new_v4(),
        tenant_id,
        external_id: None,
        role: Role::Agent,
        name: "console-test-agent".into(),
        agent: Some(Agent {
            status: AgentStatus::Offline,
            description: "tests atomic socket presence".into(),
            secret: "test-secret".into(),
            instances: 0,
            skills: Vec::new(),
        }),
        metadata: types::data::Metadata::new(),
        embedding: None,
        created_at: now,
        updated_at: now,
    }
}

fn user(tenant_id: uuid::Uuid) -> Actor {
    let now = chrono::Utc::now();
    Actor {
        id: uuid::Uuid::new_v4(),
        tenant_id,
        external_id: Some(uuid::Uuid::new_v4().to_string()),
        role: Role::User,
        name: "console-test-user".into(),
        agent: None,
        metadata: types::data::Metadata::new(),
        embedding: None,
        created_at: now,
        updated_at: now,
    }
}

fn event(tenant_id: uuid::Uuid, trace_id: uuid::Uuid, chat_id: uuid::Uuid) -> types::events::Event {
    types::events::new(
        tenant_id,
        trace_id,
        "chat.members",
        ChatMembers {
            chat_id,
            actor_ids: Vec::new(),
        },
    )
}

#[sqlx::test]
async fn event_replay_is_tenant_scoped(pool: PgPool) {
    let tenant_id = uuid::Uuid::new_v4();
    let other_tenant_id = uuid::Uuid::new_v4();
    let trace_id = uuid::Uuid::new_v4();
    let events = EventStorage::new(&pool);

    let first = events
        .create(None, None, None, None, event(tenant_id, trace_id, uuid::Uuid::new_v4()))
        .await
        .unwrap();
    events
        .create(None, None, None, None, event(other_tenant_id, trace_id, uuid::Uuid::new_v4()))
        .await
        .unwrap();
    let second = events
        .create(None, None, None, None, event(tenant_id, trace_id, uuid::Uuid::new_v4()))
        .await
        .unwrap();

    let replay = events
        .list_after(tenant_id, Some(EventCursor::from(&first)), 50)
        .await
        .unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].id, second.id);
}

#[sqlx::test]
async fn concurrent_connections_keep_presence_counts_atomic(pool: PgPool) {
    let actor_storage = ActorStorage::new(&pool);
    let actor = actor_storage.create(agent(uuid::Uuid::new_v4())).await.unwrap();

    let (left, right) = tokio::join!(actor_storage.connect_agent(actor.id), actor_storage.connect_agent(actor.id));
    left.unwrap();
    right.unwrap();

    let connected = actor_storage.get_by_id(actor.id).await.unwrap().unwrap();
    let connected_agent = connected.agent.unwrap();
    assert_eq!(connected_agent.instances, 2);
    assert_eq!(connected_agent.status, AgentStatus::Online);

    let (left, right) = tokio::join!(
        actor_storage.disconnect_agent(actor.id),
        actor_storage.disconnect_agent(actor.id)
    );
    left.unwrap();
    right.unwrap();

    let disconnected = actor_storage.get_by_id(actor.id).await.unwrap().unwrap();
    let disconnected_agent = disconnected.agent.unwrap();
    assert_eq!(disconnected_agent.instances, 0);
    assert_eq!(disconnected_agent.status, AgentStatus::Offline);
}

#[sqlx::test]
async fn existing_chat_requires_matching_tenant_and_membership(pool: PgPool) {
    let actors = ActorStorage::new(&pool);
    let chats = ChatStorage::new(&pool);
    let tenant_id = uuid::Uuid::new_v4();
    let other_tenant_id = uuid::Uuid::new_v4();
    let creator = actors.create(user(tenant_id)).await.unwrap();
    let outsider = actors.create(user(tenant_id)).await.unwrap();
    let now = chrono::Utc::now();
    let chat = chats
        .create(types::chats::Chat {
            id: uuid::Uuid::new_v4(),
            tenant_id,
            name: Some("continued conversation".into()),
            created_by: creator.clone().into(),
            created_at: now,
            updated_at: now,
            closed_at: None,
        })
        .await
        .unwrap();
    chats.set_actors(chat.id, [creator.id]).await.unwrap();

    assert!(
        chats
            .get_open_for_actor(chat.id, tenant_id, creator.id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        chats
            .get_open_for_actor(chat.id, tenant_id, outsider.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        chats
            .get_open_for_actor(chat.id, other_tenant_id, creator.id)
            .await
            .unwrap()
            .is_none()
    );
}
