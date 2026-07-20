#![allow(unused)]

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use storage::Storage;

#[derive(Clone)]
pub struct Context<'a> {
    pool: &'a PgPool,
    socket: &'a amqp::Socket,
    start_time: DateTime<Utc>,
}

impl<'a> Context<'a> {
    pub fn new(pool: &'a PgPool, socket: &'a amqp::Socket) -> Self {
        Self {
            pool,
            socket,
            start_time: Utc::now(),
        }
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn storage(&self) -> Storage<'_> {
        Storage::new(self.pool)
    }

    pub fn pool(&self) -> &PgPool {
        self.pool
    }
}

#[derive(Clone)]
pub struct EventContext<'a> {
    ctx: &'a Context<'a>,
    delivery: &'a amqp::lapin::message::Delivery,
    event: &'a types::events::Event,
}

impl<'a> EventContext<'a> {
    pub fn new(ctx: &'a Context, delivery: &'a amqp::lapin::message::Delivery, event: &'a types::events::Event) -> Self {
        Self { ctx, delivery, event }
    }

    pub fn event(&self) -> &types::events::Event {
        self.event
    }

    pub async fn ack(&self) -> Result<(), amqp::AMQPError> {
        Ok(self.delivery.ack(amqp::lapin::options::BasicAckOptions::default()).await?)
    }

    pub async fn enqueue(
        &self,
        key: impl std::fmt::Display,
        body: impl Into<types::events::Data>,
    ) -> Result<(), amqp::AMQPError> {
        self.socket
            .produce()
            .enqueue(types::events::new(self.event().trace_id, key, body))
            .await
    }

    pub async fn nack(&self) -> Result<(), amqp::AMQPError> {
        Ok(self
            .delivery
            .nack(amqp::lapin::options::BasicNackOptions {
                multiple: false,
                requeue: true,
            })
            .await?)
    }
}

impl<'a> std::ops::Deref for EventContext<'a> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
