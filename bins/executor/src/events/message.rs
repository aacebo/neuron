use amqp::{Action, Key};
use storage::rows::{JobSource, Task};

use crate::context::EventContext;

pub async fn on_create<'a>(ctx: EventContext<'a, storage::rows::Message>) -> Result<(), Box<dyn std::error::Error>> {
    let msg = &ctx.event().body;
    let storage = ctx.storage();
    let task = storage
        .tasks()
        .create(&Task::new("inference"), JobSource::message(msg.id))
        .await?;

    ctx.trace("message.create").await?;
    ctx.enqueue(Key::new("task", Action::Create), task).await?;
    ctx.ack().await?;

    Ok(())
}
