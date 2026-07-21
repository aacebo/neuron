use error::{Error, Result};
use storage::rows::{Annotation, Artifact, ArtifactContent, Span, Task, TextArtifact};

type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use crate::context::EventContext;

pub async fn on_attempt<'a>(ctx: EventContext<'a, Task>) -> BoxResult<()> {
    let storage = ctx.storage();
    let task = match storage.tasks().get(ctx.event().body.id).await? {
        Some(j) => j,
        None => {
            ctx.ack().await?;
            return Ok(());
        }
    };

    if task.attempts >= task.max_attempts {
        ctx.ack().await?;
        return Ok(());
    }

    ctx.trace("task.start").await?;
    let task = task.start();
    storage.tasks().update(&task).await?;

    let messages = storage.messages().get_by_task(task.id).await?;
    let text = messages.first().map(|m| m.text.clone()).unwrap_or_default();
    let output = tokio::task::block_in_place(|| {
        let text = vec![text.clone()];
        let model = || ai::pipelines::ModelArgs {
            provider: None,
            model: None,
            base_url: None,
            api_key: None,
        };
        let scored = || ai::pipelines::ScoredArgs {
            text: text.clone(),
            min_score: 0.4,
            model: model(),
        };

        std::thread::scope(|scope| {
            let keywords = scope.spawn(|| ai::pipelines::keywords(scored()));
            let sentiment = scope.spawn(|| ai::pipelines::sentiment(scored()));
            let entities = scope.spawn(|| ai::pipelines::entities(scored()));
            let pii = scope.spawn(|| ai::pipelines::pii(scored()));
            let embeddings = scope.spawn(|| {
                ai::pipelines::embeddings(ai::pipelines::TextArgs {
                    text: text.clone(),
                    model: model(),
                })
            });
            let summary = scope.spawn(|| {
                ai::pipelines::summarize(ai::pipelines::SummarizeArgs {
                    text: text.clone(),
                    model: model(),
                    beams: Some(1),
                    max_len: Some(64),
                })
            });

            fn join<T>(handle: std::thread::ScopedJoinHandle<'_, Result<T>>) -> Result<T> {
                match handle.join() {
                    Ok(result) => result,
                    Err(_) => Err(error::ai::inference("inference thread panicked")),
                }
            }

            let mut annotations = join(keywords)?;
            annotations.extend(join(sentiment)?);
            annotations.extend(join(entities)?);

            for entity in join(pii)?.into_iter().flatten() {
                annotations.push(ai::types::Annotation {
                    name: "pii".to_string(),
                    label: entity.label.to_lowercase(),
                    text: entity.word,
                    score: entity.score,
                    spans: vec![entity.offset],
                });
            }

            let mut artifacts = join(embeddings)?;
            artifacts.extend(join(summary)?);
            Ok::<_, Error>((annotations, artifacts))
        })
    });

    let (annotations, artifacts) = match output {
        Ok(o) => o,
        Err(e) => {
            let task = task.fail(e.to_string());
            storage.tasks().update(&task).await?;

            if task.attempts >= task.max_attempts {
                ctx.ack().await?;
            } else {
                ctx.nack().await?;
            }

            return Ok(());
        }
    };

    let message_id = messages.first().map(|m| m.id).unwrap();
    let persist_result: BoxResult<()> = async {
        for annotation in &annotations {
            let spans = annotation.spans.iter().map(|s| Span::new(s.begin, s.end)).collect();
            storage
                .annotations()
                .create(&Annotation::new(
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
            let row = Artifact::new(
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
            ctx.trace("task.success").await?;
            storage.tasks().update(&task.success()).await?;
            ctx.ack().await?;
        }
        Err(e) => {
            ctx.error("task.fail", task.id.to_string()).await?;
            let task = task.clone().fail(e.to_string());
            storage.tasks().update(&task).await?;

            if task.attempts >= task.max_attempts {
                ctx.ack().await?;
            } else {
                ctx.nack().await?;
            }
        }
    }

    Ok(())
}
