use std::collections::HashMap;
use std::sync::Arc;

use error::Result;

use crate::{BindingKey, SocketConsumer, SocketProducer};

pub const EVENTS_EXCHANGE: &str = "events";

#[derive(Clone)]
pub struct Socket {
    app_id: String,
    conn: Arc<lapin::Connection>,
    channel: Arc<lapin::Channel>,
    queues: HashMap<String, lapin::Queue>,
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

    pub fn queue(&self, name: &str) -> Option<&lapin::Queue> {
        self.queues.get(name)
    }

    pub async fn consume(&self, queue_name: &str) -> Result<SocketConsumer<'_>> {
        let queue = self
            .queue(queue_name)
            .ok_or_else(|| error::amqp(format!("queue {queue_name} not found")))?;
        let consumer_tag = format!("{}::{queue_name}", self.app_id());
        let consumer = self
            .channel()
            .basic_consume(
                queue.name().as_str(),
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

#[derive(Debug, Clone)]
pub struct QueueOptions {
    name: String,
    bindings: Vec<BindingKey>,
}

impl QueueOptions {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bindings: vec![],
        }
    }

    pub fn with_binding(mut self, key: BindingKey) -> Self {
        self.bindings.push(key);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bindings(&self) -> &[BindingKey] {
        &self.bindings
    }
}

pub struct SocketOptions {
    app_id: String,
    uri: String,
    queues: Vec<QueueOptions>,
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

    pub fn with_queue(mut self, options: QueueOptions) -> Self {
        self.queues.push(options);
        self
    }

    pub async fn connect(self) -> Result<Socket> {
        let conn = lapin::Connection::connect(&self.uri, lapin::ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        let mut queues = HashMap::new();

        channel
            .exchange_declare(
                EVENTS_EXCHANGE,
                lapin::ExchangeKind::Topic,
                lapin::options::ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await?;

        for options in self.queues {
            if options.name().is_empty() {
                return Err(error::amqp("queue name cannot be empty"));
            }

            if options.bindings().is_empty() {
                return Err(error::amqp(format!(
                    "queue {} must have at least one binding",
                    options.name()
                )));
            }

            if queues.contains_key(options.name()) {
                return Err(error::amqp(format!("queue {} configured more than once", options.name())));
            }

            let queue = channel
                .queue_declare(
                    options.name(),
                    lapin::options::QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    lapin::types::FieldTable::default(),
                )
                .await?;

            for binding in options.bindings() {
                channel
                    .queue_bind(
                        queue.name().as_str(),
                        EVENTS_EXCHANGE,
                        &binding.to_string(),
                        lapin::options::QueueBindOptions::default(),
                        lapin::types::FieldTable::default(),
                    )
                    .await?;
            }

            queues.insert(options.name, queue);
        }

        Ok(Socket {
            app_id: self.app_id,
            conn: Arc::new(conn),
            channel: Arc::new(channel),
            queues,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::QueueOptions;

    #[test]
    fn queue_options_keep_name_and_multiple_bindings() {
        let options = QueueOptions::new("neuron.worker.events")
            .with_binding("actor.*".parse().unwrap())
            .with_binding("message.inbound".parse().unwrap());

        assert_eq!(options.name(), "neuron.worker.events");
        assert_eq!(
            options.bindings().iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["actor.*", "message.inbound"]
        );
    }
}
