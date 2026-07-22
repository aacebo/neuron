use crate::context::EventContext;

mod inbound;

pub async fn on_event(ctx: EventContext<'_>) -> ::error::Result<()> {
    match ctx.event().key.as_str() {
        "message.inbound" => {},
        _ => {},
    };

    Ok(())
}
