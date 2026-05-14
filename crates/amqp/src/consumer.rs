use futures_lite::StreamExt;

use crate::{AMQPError, Event, Socket};

#[derive(Clone)]
pub struct SocketConsumer<'a> {
    socket: &'a Socket,
    consumer: lapin::Consumer,
}

impl<'a> SocketConsumer<'a> {
    pub(crate) fn new(socket: &'a Socket, consumer: lapin::Consumer) -> Self {
        Self { socket, consumer }
    }

    pub fn socket(&self) -> &'a Socket {
        self.socket
    }

    pub async fn dequeue<T: for<'b> serde::Deserialize<'b>>(
        &mut self,
    ) -> Option<Result<(lapin::message::Delivery, Event<T>), AMQPError>> {
        let delivery = match self.consumer.next().await? {
            Err(err) => return Some(Err(err.into())),
            Ok(v) => v,
        };

        let data: Event<T> = match serde_json::from_slice(&delivery.data) {
            Err(err) => return Some(Err(err.into())),
            Ok(v) => v,
        };

        Some(Ok((delivery, data)))
    }
}
