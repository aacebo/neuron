use storage::types::{ArtifactContent, Job, MessageAnnotation, MessageArtifact, Span, TextArtifact};

use crate::context::EventContext;

pub async fn on_attempt<'a>(ctx: EventContext<'a, Job>) -> Result<(), Box<dyn std::error::Error>> {
    let storage = ctx.storage();
    let job = match storage.jobs().get(ctx.event().body.id).await? {
        Some(j) => j,
        None => {
            ctx.ack().await?;
            return Ok(());
        }
    };

    if job.attempts >= job.max_attempts {
        ctx.ack().await?;
        return Ok(());
    }

    let job = job.start();
    storage.jobs().update(&job).await?;

    let messages = storage.messages().get_by_job(job.id).await?;
    let text = messages.first().map(|m| m.text.clone()).unwrap_or_default();
    let output = tokio::task::block_in_place(|| {
        let text = vec![text.clone()];
        let model = || ai::pipelines::ModelArgs {
            provider: None,
            model: None,
            base_url: None,
            api_key: None,
        };
        let text_args = || ai::pipelines::TextArgs {
            text: text.clone(),
            model: model(),
        };
        let scored = || ai::pipelines::ScoredArgs {
            text: text.clone(),
            min_score: 0.4,
            model: model(),
        };

        let mut annotations = ai::pipelines::keywords(scored())?;
        annotations.extend(ai::pipelines::sentiment(scored())?);
        annotations.extend(ai::pipelines::entities(scored())?);

        // pii returns entities, not annotations
        for entity in ai::pipelines::pii(scored())?.into_iter().flatten() {
            annotations.push(ai::types::Annotation {
                name: "pii".to_string(),
                label: entity.label.to_lowercase(),
                text: entity.word,
                score: entity.score,
                spans: vec![entity.offset],
            });
        }

        let mut artifacts = ai::pipelines::embeddings(text_args())?;
        artifacts.extend(ai::pipelines::summarize(text_args())?);
        Ok::<_, Box<dyn std::error::Error>>((annotations, artifacts))
    });

    let (annotations, artifacts) = match output {
        Ok(o) => o,
        Err(e) => {
            let job = job.fail(e.to_string());
            storage.jobs().update(&job).await?;

            if job.attempts >= job.max_attempts {
                ctx.ack().await?;
            } else {
                ctx.nack().await?;
            }

            return Ok(());
        }
    };

    let message_id = messages.first().map(|m| m.id).unwrap();
    let persist_result: Result<(), Box<dyn std::error::Error>> = async {
        for annotation in &annotations {
            let spans = annotation.spans.iter().map(|s| Span::new(s.begin, s.end)).collect();
            storage
                .annotations()
                .create(&MessageAnnotation::new(
                    message_id,
                    &annotation.name,
                    &annotation.label,
                    &annotation.text,
                    annotation.score,
                    spans,
                ))
                .await?;
        }

        for artifact in artifacts {
            let Some(text) = artifact.value.as_text() else { continue };
            let row = MessageArtifact::new(
                message_id,
                artifact.name.clone(),
                ArtifactContent::Text(TextArtifact { text: text.to_string() }),
                artifact.vector.clone(),
            );

            storage.artifacts().create(&row).await?;
        }

        Ok(())
    }
    .await;

    match persist_result {
        Ok(()) => {
            storage.jobs().update(&job.success()).await?;
            ctx.ack().await?;
        }
        Err(e) => {
            let job = job.clone().fail(e.to_string());
            storage.jobs().update(&job).await?;

            if job.attempts >= job.max_attempts {
                ctx.ack().await?;
            } else {
                ctx.nack().await?;
            }
        }
    }

    Ok(())
}
