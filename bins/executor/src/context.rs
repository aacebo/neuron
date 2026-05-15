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
