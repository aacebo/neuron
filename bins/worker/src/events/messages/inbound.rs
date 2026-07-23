use crate::context::EventContext;

pub async fn run(ctx: &EventContext<'_>, data: &types::chats::InboundMessage) -> error::Result<()> {
    tracing::debug!(?data, "message.inbound");
    ctx.ack().await?;
    Ok(())
}
