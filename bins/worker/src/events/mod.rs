use crate::context::EventContext;

pub mod actors;
pub mod messages;

pub async fn run(ctx: &EventContext<'_>) -> error::Result<()> {
    match (ctx.event().key.as_str(), &ctx.event().data) {
        ("actor.create" | "actor.update", types::events::Data::Actor { actor }) => actors::run(ctx, actor.id).await,
        ("message.inbound", types::events::Data::InboundMessage { message: _ }) => messages::run(ctx).await,
        (key, data) => {
            tracing::info!(key, ?data, "unsupported event");
            ctx.reject().await?;
            Ok(())
        }
    }
}
