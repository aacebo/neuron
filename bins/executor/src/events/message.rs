use amqp::{Action, Event, Key};
use storage::types::{Job, JobSource};

use crate::context::EventContext;

pub async fn on_create<'a>(ctx: EventContext<'a, storage::types::Message>) -> Result<(), Box<dyn std::error::Error>> {
    let msg = &ctx.event().body;
    let storage = ctx.storage();
    let job = storage
        .jobs()
        .create(&Job::new("inference"), JobSource::message(msg.id))
        .await?;

    ctx.amqp()
        .produce()
        .enqueue(Event::new(Key::new("job", Action::Create), job))
        .await?;

    ctx.ack().await?;
    Ok(())
}
