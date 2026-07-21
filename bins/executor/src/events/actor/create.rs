use crate::context::EventContext;

pub async fn on_create(ctx: EventContext<'_>, actor: &types::actors::Actor) -> Result<(), Box<dyn std::error::Error>> {
    let actor_id = actor.id;
    let result: Result<Option<usize>, Box<dyn std::error::Error>> = async {
        let actor = match ctx.storage().actors().get_by_id(actor_id).await? {
            Some(actor) => actor,
            None => {
                tracing::debug!(reason = "actor_missing", "skipping actor embedding");
                return Ok(None);
            }
        };

        if !actor.role.is_agent() {
            tracing::debug!(reason = "not_agent", "skipping actor embedding");
            return Ok(None);
        }

        if actor.embedding.is_some() {
            tracing::debug!(reason = "already_embedded", "skipping actor embedding");
            return Ok(None);
        }

        let agent = match &actor.agent {
            Some(agent) => agent,
            None => {
                tracing::debug!(reason = "agent_details_missing", "skipping actor embedding");
                return Ok(None);
            }
        };

        let mut lines = vec![
            format!("Name: {}", actor.name),
            format!("Display name: {}", actor.display_name),
            format!("Description: {}", agent.description),
        ];

        if !agent.skills.is_empty() {
            lines.push("Skills:".to_string());

            for skill in &agent.skills {
                lines.push(format!("- Name: {}", skill.name));
                lines.push(format!("  Display name: {}", skill.display_name));

                if let Some(description) = &skill.description {
                    lines.push(format!("  Description: {description}"));
                }
            }
        }

        let input = lines.join("\n");
        let artifacts = tokio::task::block_in_place(move || {
            ai::pipelines::embeddings(ai::pipelines::TextArgs {
                text: vec![input],
                model: ai::pipelines::ModelArgs {
                    provider: None,
                    model: None,
                    base_url: None,
                    api_key: None,
                },
            })
        })?;

        if artifacts.len() != 1 {
            return Err(std::io::Error::other(format!(
                "embedding pipeline returned {} artifacts; expected 1",
                artifacts.len()
            ))
            .into());
        }

        let vector = artifacts
            .into_iter()
            .next()
            .and_then(|artifact| artifact.vector)
            .ok_or_else(|| std::io::Error::other("embedding pipeline returned no vector"))?;

        if vector.len() != 384 {
            return Err(std::io::Error::other(format!(
                "embedding pipeline returned {} dimensions; expected 384",
                vector.len()
            ))
            .into());
        }

        let dimensions = vector.len();
        ctx.storage().actors().update_embedding(actor_id, vector).await?;
        Ok(Some(dimensions))
    }
    .await;

    match result {
        Ok(None) => ctx.ack().await?,
        Ok(Some(dimensions)) => {
            tracing::info!(dimensions, "stored agent embedding");
            ctx.ack().await?;
        }
        Err(error) => {
            tracing::error!(%error, "failed to create agent embedding; requeuing event");
            ctx.nack().await?;
        }
    }

    Ok(())
}
