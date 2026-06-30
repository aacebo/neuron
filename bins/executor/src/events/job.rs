use cortex::types::CortexArtifact;
use cortex::{CortexInput, Routine};
use storage::types::{
    ArtifactContent, Job, MessageAnnotation, MessageArtifact, Span, TextArtifact,
};

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
        let binding = [text.as_str()];
        let input = CortexInput::new(&binding).with_min_score(0.4);
        let out = ctx.cortex().pipeline().invoke(input)?;
        Ok::<_, cortex::CortexError>(out)
    });

    let pipeline_result: Result<cortex::CortexOutput, Box<dyn std::error::Error>> =
        output.map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
    let output = match pipeline_result {
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
        for annotation in &output.annotations {
            let spans = annotation
                .spans
                .iter()
                .map(|s| Span::new(s.start, s.end))
                .collect();
            storage
                .annotations()
                .create(&MessageAnnotation::new(
                    message_id,
                    &annotation.r#type,
                    &annotation.label,
                    &annotation.text,
                    annotation.score,
                    spans,
                ))
                .await?;
        }

        for artifact in output.artifacts {
            let row = match artifact {
                CortexArtifact::Text(s) => MessageArtifact::new(
                    message_id,
                    s.name,
                    ArtifactContent::Text(TextArtifact {
                        text: s.text.clone(),
                    }),
                    s.vector,
                ),
            };

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
