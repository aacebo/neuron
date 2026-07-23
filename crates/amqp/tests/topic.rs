use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amqp::lapin::{options, protocol};

const TEST_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
#[ignore = "requires a disposable RabbitMQ instance"]
async fn topic_bindings_route_to_one_shared_queue() {
    let uri = std::env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://admin:admin@localhost:5672".to_string());
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let queue_name = format!("neuron.amqp.test.{suffix}");
    let queue = || {
        amqp::QueueOptions::new(&queue_name)
            .with_binding("entity.*".parse().unwrap())
            .with_binding("message.inbound".parse().unwrap())
    };

    let first = amqp::new(&uri)
        .with_app_id("neuron::amqp-test-a")
        .with_queue(queue())
        .connect()
        .await
        .unwrap();

    publish(first.channel(), "entity.create", &event_payload("entity.create", 1)).await;
    publish(first.channel(), "actor.create", &event_payload("actor.create", 2)).await;
    publish(first.channel(), "entity.child.create", &event_payload("entity.create", 3)).await;
    publish(first.channel(), "message.inbound", &event_payload("message.inbound", 4)).await;

    let first_message = get(&first, &queue_name).await.unwrap();
    let second_message = get(&first, &queue_name).await.unwrap();
    assert_eq!(first_message.delivery.routing_key.as_str(), "entity.create");
    assert_eq!(second_message.delivery.routing_key.as_str(), "message.inbound");
    assert!(get(&first, &queue_name).await.is_none());

    let second = amqp::new(&uri)
        .with_app_id("neuron::amqp-test-b")
        .with_queue(queue())
        .connect()
        .await
        .unwrap();
    let mut consumer_a = first.consume(&queue_name).await.unwrap();
    let mut consumer_b = second.consume(&queue_name).await.unwrap();

    publish(first.channel(), "entity.create", &event_payload("entity.create", 5)).await;
    publish(first.channel(), "entity.update", &event_payload("entity.update", 6)).await;

    let (delivery_a, event_a) = tokio::time::timeout(TEST_TIMEOUT, consumer_a.dequeue())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let (delivery_b, event_b) = tokio::time::timeout(TEST_TIMEOUT, consumer_b.dequeue())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_ne!(event_a.id, event_b.id);
    delivery_a.ack(options::BasicAckOptions::default()).await.unwrap();
    delivery_b.ack(options::BasicAckOptions::default()).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(200), consumer_a.dequeue())
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(200), consumer_b.dequeue())
            .await
            .is_err()
    );

    drop(consumer_a);
    drop(consumer_b);

    publish(first.channel(), "entity.delete", &event_payload("entity.delete", 7)).await;
    let unsupported = first
        .channel()
        .basic_get(&queue_name, options::BasicGetOptions { no_ack: false })
        .await
        .unwrap()
        .unwrap();
    unsupported
        .delivery
        .reject(options::BasicRejectOptions { requeue: false })
        .await
        .unwrap();
    assert!(get(&first, &queue_name).await.is_none());

    first
        .channel()
        .queue_delete(&queue_name, options::QueueDeleteOptions::default())
        .await
        .unwrap();
}

async fn publish(channel: &amqp::lapin::Channel, key: &str, payload: &[u8]) {
    channel
        .basic_publish(
            amqp::EVENTS_EXCHANGE,
            key,
            options::BasicPublishOptions::default(),
            payload,
            protocol::basic::AMQPProperties::default().with_content_type("application/json".into()),
        )
        .await
        .unwrap()
        .await
        .unwrap();
}

async fn get(socket: &amqp::Socket, queue_name: &str) -> Option<amqp::lapin::message::BasicGetMessage> {
    socket
        .channel()
        .basic_get(queue_name, options::BasicGetOptions { no_ack: true })
        .await
        .unwrap()
}

fn event_payload(key: &str, id: u128) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "id": format!("00000000-0000-0000-0000-{id:012x}"),
        "tenant_id": "00000000-0000-0000-0000-000000000001",
        "trace_id": "00000000-0000-0000-0000-000000000002",
        "key": key,
        "data": {
            "type": "actor",
            "actor": {
                "id": "00000000-0000-0000-0000-000000000003",
                "tenant_id": "00000000-0000-0000-0000-000000000001",
                "external_id": null,
                "role": "user",
                "name": "AMQP test actor",
                "metadata": {},
                "created_at": "2026-07-22T00:00:00Z",
                "updated_at": "2026-07-22T00:00:00Z"
            }
        },
        "created_at": "2026-07-22T00:00:00Z"
    }))
    .unwrap()
}
