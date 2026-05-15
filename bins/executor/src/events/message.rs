use storage::types::Message;

use crate::context::EventContext;

pub async fn on_create<'a>(
    ctx: EventContext<'a, Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{:#?}", ctx.event().body);
    Ok(())
}
