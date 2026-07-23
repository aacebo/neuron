use crate::context::EventContext;

pub async fn run(ctx: &EventContext<'_>, data: &types::chats::InboundMessage) -> error::Result<()> {
    tracing::trace!(?data, "message.inbound");
    ctx.ack().await?;
    Ok(())
}
