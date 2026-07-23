use crate::context::EventContext;

pub async fn run(ctx: &EventContext<'_>, message: &types::chats::InboundMessage) -> error::Result<()> {
    let result: error::Result<Option<usize>> = async {
        let artifacts = tokio::task::block_in_place(move || {
            ai::pipelines::embeddings(ai::pipelines::TextArgs {
                text: vec![message.content.to_string()],
                ..Default::default()
            })
        })?;

        if artifacts.len() != 1 {
            return Err(error::ai(format!(
                "embedding pipeline returned {} artifacts; expected 1",
                artifacts.len()
            )));
        }

        let vector = artifacts
            .into_iter()
            .next()
            .and_then(|artifact| artifact.vector)
            .ok_or_else(|| error::ai("embedding pipeline returned no vector"))?;

        let dimensions = vector.len();
        let matches = ctx
            .storage()
            .actors()
            .search(ctx.event().tenant_id, vector.clone(), Default::default())
            .await?;

        if matches.is_empty() {
            tracing::debug!(?message, "no actors were found");
            return Ok(None);
        }

        let chat = ctx
            .storage()
            .chats()
            .create(types::chats::Chat {
                id: uuid::Uuid::new_v4(),
                tenant_id: message.tenant_id,
                name: message.subject.clone(),
                created_by: message.sent_by.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                closed_at: None,
            })
            .await?;

        ctx.enqueue("chat.create", chat.clone()).await?;
        let mut member_ids = vec![message.sent_by.id];
        member_ids.extend(matches.iter().map(|m| m.entity.id));
        ctx.storage().chats().set_actors(chat.id, member_ids).await?;
        let message = ctx
            .storage()
            .messages()
            .create(types::chats::Message {
                id: uuid::Uuid::new_v4(),
                chat: chat.into(),
                content: message.content.clone(),
                metadata: message.metadata.clone(),
                embedding: Some(vector),
                created_by: message.sent_by.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .await?;

        ctx.enqueue("message.create", message).await?;
        Ok(Some(dimensions))
    }
    .await;

    match result {
        Ok(None) => ctx.ack().await?,
        Ok(Some(_)) => ctx.ack().await?,
        Err(error) => {
            tracing::error!(%error, "failed to create agent embedding; requeuing event");
            ctx.nack().await?;
        }
    }

    Ok(())
}
