use lapin::{options, protocol};

use crate::{AMQPError, Key, Socket};

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

    pub async fn enqueue(&self, event: types::events::Event) -> Result<(), AMQPError> {
        let key = event.key.parse::<Key>()?;
        let payload = serde_json::to_vec(&event)?;
        let routing_key = key.to_string();

        self.socket
            .channel()
            .basic_publish(
                key.exchange(),
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
