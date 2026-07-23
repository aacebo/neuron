use crate::context::EventContext;

pub async fn run(_ctx: &EventContext<'_>, data: &types::chats::InboundMessage) -> error::Result<()> {
    tracing::trace!(?data, "message.inbound");
    Ok(())
}
