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
pub struct EventContext<'a, T> {
    ctx: &'a Context<'a>,
    delivery: &'a amqp::lapin::message::Delivery,
    event: &'a amqp::Event<T>,
}

impl<'a, T> EventContext<'a, T> {
    pub fn new(ctx: &'a Context, delivery: &'a amqp::lapin::message::Delivery, event: &'a amqp::Event<T>) -> Self {
        Self { ctx, delivery, event }
    }

    pub fn event(&self) -> &amqp::Event<T> {
        self.event
    }

    pub async fn ack(&self) -> Result<(), amqp::AMQPError> {
        Ok(self.delivery.ack(amqp::lapin::options::BasicAckOptions::default()).await?)
    }

    pub async fn enqueue<V: serde::Serialize>(&self, key: impl Into<amqp::Key>, body: V) -> Result<(), amqp::AMQPError> {
        self.socket
            .produce()
            .enqueue(amqp::Event::new(self.event().trace_id, key.into(), body))
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

    pub async fn trace(&self, message: impl std::fmt::Display) -> Result<(), amqp::AMQPError> {
        self.enqueue(
            amqp::Key::new("log", amqp::Action::Create),
            storage::rows::Log::trace(self.event.trace_id, "executor", message),
        )
        .await
    }

    pub async fn info(&self, message: impl std::fmt::Display) -> Result<(), amqp::AMQPError> {
        self.enqueue(
            amqp::Key::new("log", amqp::Action::Create),
            storage::rows::Log::info(self.event.trace_id, "executor", message),
        )
        .await
    }

    pub async fn warn(&self, message: impl std::fmt::Display) -> Result<(), amqp::AMQPError> {
        self.enqueue(
            amqp::Key::new("log", amqp::Action::Create),
            storage::rows::Log::warn(self.event.trace_id, "executor", message),
        )
        .await
    }

    pub async fn error(
        &self,
        message: impl std::fmt::Display,
        context: impl Into<sqlx::types::JsonValue>,
    ) -> Result<(), amqp::AMQPError> {
        self.enqueue(
            amqp::Key::new("log", amqp::Action::Create),
            storage::rows::Log::error(self.event.trace_id, "executor", message).with(context),
        )
        .await
    }
}

impl<'a, T> std::ops::Deref for EventContext<'a, T> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
