use crate::context::EventContext;

pub async fn on_create<'a>(ctx: EventContext<'a, storage::types::Log>) -> Result<(), Box<dyn std::error::Error>> {
    let log = &ctx.event().body;
    let storage = ctx.storage();
    let log = storage.logs().create(&log).await?;

    println!("{}", log.message);
    ctx.ack().await?;
    Ok(())
}
