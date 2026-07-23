use crate::context::EventContext;

mod inbound;

pub async fn run(ctx: &EventContext<'_>) -> error::Result<()> {
    match (ctx.event().key.as_str(), &ctx.event().data) {
        ("message.inbound", types::events::Data::InboundMessage { message }) => inbound::run(ctx, message).await,
        (key, data) => Err(error::bad_request(format!("unsupported event {} => {:#?}", key, data)).trace(ctx.event().trace_id)),
    }
}
