use lapin::{options, protocol};

use crate::{AMQPError, Event, Socket};

#[derive(Clone)]
pub struct SocketProducer<'a> {
    socket: &'a Socket,
}

impl<'a> SocketProducer<'a> {
    pub(crate) fn new(socket: &'a Socket) -> Self {
        Self { socket }
    }

    pub fn socket(&self) -> &'a Socket {
        self.socket
    }

    pub async fn enqueue<TBody: serde::Serialize>(
        &self,
        event: Event<TBody>,
    ) -> Result<(), AMQPError> {
        let payload = serde_json::to_vec(&event)?;
        // Routing key must match the consumer's queue bind key
        // (socket.rs binds with `key.to_string()` = "<entity>.<action>").
        // `basic_publish`'s 2nd arg is the routing key, not a queue name.
        let routing_key = event.key.to_string();
        self.socket
            .channel()
            .basic_publish(
                event.key.exchange(),
                &routing_key,
                options::BasicPublishOptions::default(),
                &payload,
                protocol::basic::AMQPProperties::default()
                    .with_app_id(self.socket().app_id().into())
                    .with_content_type("application/json".into()),
            )
            .await?;

        Ok(())
    }
}
