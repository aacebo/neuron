#![allow(unused)]

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use storage::Storage;

#[derive(Clone)]
pub struct Context {
    pool: PgPool,
    socket: amqp::Socket,
    start_time: DateTime<Utc>,
}

impl Context {
    pub fn new(pool: PgPool, socket: amqp::Socket) -> Self {
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
        Storage::new(&self.pool)
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn amqp(&self) -> &amqp::Socket {
        &self.socket
    }
}

#[derive(Clone)]
pub struct EventContext<'a, T> {
    ctx: &'a Context,
    delivery: &'a amqp::lapin::message::Delivery,
    event: &'a amqp::Event<T>,
}

impl<'a, T> EventContext<'a, T> {
    pub fn new(
        ctx: &'a Context,
        delivery: &'a amqp::lapin::message::Delivery,
        event: &'a amqp::Event<T>,
    ) -> Self {
        Self {
            ctx,
            delivery,
            event,
        }
    }

    pub fn event(&self) -> &amqp::Event<T> {
        self.event
    }

    pub async fn ack(&self) -> Result<(), amqp::AMQPError> {
        Ok(self
            .delivery
            .ack(amqp::lapin::options::BasicAckOptions::default())
            .await?)
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

impl<'a, T> std::ops::Deref for EventContext<'a, T> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
