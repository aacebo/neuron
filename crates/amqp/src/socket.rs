use std::collections::HashMap;
use std::sync::Arc;

use crate::{Error, Key, Result, SocketConsumer, SocketProducer};

#[derive(Clone)]
pub struct Socket {
    app_id: String,
    conn: Arc<lapin::Connection>,
    channel: Arc<lapin::Channel>,
    queues: HashMap<Key, lapin::Queue>,
}

impl Socket {
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn conn(&self) -> &lapin::Connection {
        &self.conn
    }

    pub fn channel(&self) -> &lapin::Channel {
        &self.channel
    }

    pub fn queue(&self, key: Key) -> Option<&lapin::Queue> {
        self.queues.get(&key)
    }

    pub async fn consume(&self, key: &str) -> Result<SocketConsumer<'_>> {
        let key = key.parse()?;

        if !self.queues.contains_key(&key) {
            return Err(Error::custom("not-found", "queue not found"));
        }

        let consumer_tag = format!("{}::{}", self.app_id(), key);
        let consumer = self
            .channel()
            .basic_consume(
                &key.to_string(),
                &consumer_tag,
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;

        Ok(SocketConsumer::new(self, consumer))
    }

    pub fn produce(&self) -> SocketProducer<'_> {
        SocketProducer::new(self)
    }
}

pub struct SocketOptions {
    app_id: String,
    uri: String,
    queues: Vec<Key>,
}

impl SocketOptions {
    pub fn new(uri: &str) -> Self {
        Self {
            app_id: String::new(),
            uri: uri.to_string(),
            queues: vec![],
        }
    }

    pub fn with_app_id(mut self, app_id: &str) -> Self {
        self.app_id = app_id.to_string();
        self
    }

    pub fn with_queue(mut self, key: Key) -> Self {
        self.queues.push(key);
        self
    }

    pub async fn connect(self) -> Result<Socket> {
        let conn = lapin::Connection::connect(&self.uri, lapin::ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        let mut queues = HashMap::new();

        for key in self.queues {
            channel
                .exchange_declare(
                    key.exchange(),
                    lapin::ExchangeKind::Direct,
                    lapin::options::ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    lapin::types::FieldTable::default(),
                )
                .await?;

            let queue = channel
                .queue_declare(
                    &key.to_string(),
                    lapin::options::QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    lapin::types::FieldTable::default(),
                )
                .await?;

            channel
                .queue_bind(
                    &key.to_string(),
                    key.exchange(),
                    &key.to_string(),
                    lapin::options::QueueBindOptions::default(),
                    lapin::types::FieldTable::default(),
                )
                .await?;

            queues.insert(key, queue);
        }

        Ok(Socket {
            app_id: self.app_id,
            conn: Arc::new(conn),
            channel: Arc::new(channel),
            queues,
        })
    }
}
