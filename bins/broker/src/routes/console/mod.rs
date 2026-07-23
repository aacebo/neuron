use std::collections::HashSet;
use std::time::Duration;

use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::{AggregatedMessage, CloseCode, CloseReason};
use askama::Template;
use futures_util::StreamExt;
use tracing::Instrument;

use crate::RequestContext;

const REPLAY_BATCH_SIZE: u32 = 250;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);

#[derive(Template)]
#[template(path = "console/page.html")]
struct ConsoleTemplate {
    config_json: String,
}

#[derive(serde::Serialize)]
struct ConsolePageConfig {
    tenant_id: uuid::Uuid,
    high_water_cursor: Option<storage::EventCursor>,
    reducer_version: u32,
}

#[derive(Debug, serde::Deserialize)]
struct ReplayQuery {
    after_at: Option<chrono::DateTime<chrono::Utc>>,
    after_id: Option<uuid::Uuid>,
}

impl ReplayQuery {
    fn cursor(&self) -> error::Result<Option<storage::EventCursor>> {
        match (self.after_at, self.after_id) {
            (Some(created_at), Some(id)) => Ok(Some(storage::EventCursor { created_at, id })),
            (None, None) => Ok(None),
            _ => Err(error::bad_request("after_at and after_id must be provided together")),
        }
    }
}

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/console")
            .service(page)
            .service(connect)
            .service(styles)
            .service(reducer)
            .service(script),
    );
}

#[get("")]
async fn page(ctx: RequestContext) -> error::Result<HttpResponse> {
    let tenant_id = ctx.console().tenant_id.unwrap();
    let config = ConsolePageConfig {
        tenant_id,
        high_water_cursor: ctx.storage().events().latest_cursor(tenant_id).await?,
        reducer_version: 2,
    };
    let template = ConsoleTemplate {
        config_json: serde_json::to_string(&config)?,
    };
    let body = template.render().map_err(error::http)?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[get("/connect")]
async fn connect(
    ctx: RequestContext,
    req: HttpRequest,
    payload: web::Payload,
    query: web::Query<ReplayQuery>,
) -> error::Result<HttpResponse> {
    let tenant_id = ctx.console().tenant_id.unwrap();
    let cursor = query.cursor()?;
    let (response, session, stream) = actix_ws::handle(&req, payload)?;
    let stream = stream.aggregate_continuations().max_continuation_size(64 * 1024);
    let span = tracing::info_span!(
        parent: ctx.span(),
        "console.connection",
        tenant_id = %tenant_id,
        replay_after_at = ?cursor.map(|cursor| cursor.created_at),
        replay_after_id = ?cursor.map(|cursor| cursor.id),
    );

    rt::spawn(run_stream(ctx, tenant_id, cursor, session, stream).instrument(span));
    Ok(response)
}

#[get("/assets/console.css")]
async fn styles() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../../../static/console.css"))
}

#[get("/assets/console-reducer.js")]
async fn reducer() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .content_type("text/javascript; charset=utf-8")
        .body(include_str!("../../../static/console-reducer.js"))
}

#[get("/assets/console.js")]
async fn script() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .content_type("text/javascript; charset=utf-8")
        .body(include_str!("../../../static/console.js"))
}

async fn run_stream(
    ctx: RequestContext,
    tenant_id: uuid::Uuid,
    cursor: Option<storage::EventCursor>,
    session: actix_ws::Session,
    stream: actix_ws::AggregatedMessageStream,
) {
    let binding = "#".parse().expect("the console event binding is valid");
    let mut events = match ctx.socket().subscribe(&[binding]).await {
        Ok(events) => events,
        Err(error) => {
            tracing::error!(%error, "failed to create console AMQP subscription");
            close(session, CloseCode::Error, "event subscription failed").await;
            return;
        }
    };

    tracing::debug!("created exclusive console AMQP subscription");
    run_event_stream(&ctx, tenant_id, cursor, session, stream, &mut events).await;

    if let Err(error) = events.cancel().await {
        tracing::warn!(%error, "failed to cancel console AMQP subscription");
    } else {
        tracing::debug!("cancelled console AMQP subscription");
    }
}

async fn run_event_stream(
    ctx: &RequestContext,
    tenant_id: uuid::Uuid,
    mut cursor: Option<storage::EventCursor>,
    mut session: actix_ws::Session,
    mut stream: actix_ws::AggregatedMessageStream,
    events: &mut amqp::SocketConsumer<'_>,
) {
    let mut sent = HashSet::new();
    let mut replayed = 0_usize;
    tracing::debug!("starting console event replay");

    loop {
        let events = match ctx.storage().events().list_after(tenant_id, cursor, REPLAY_BATCH_SIZE).await {
            Ok(events) => events,
            Err(error) => {
                tracing::error!(%error, "failed to replay console events");
                close(session, CloseCode::Error, "event replay failed").await;
                return;
            }
        };

        let count = events.len();

        for event in events {
            cursor = Some(storage::EventCursor::from(&event));
            sent.insert(event.id);

            if let Err(error) = emit(&mut session, &event).await {
                tracing::debug!(%error, event_id = %event.id, "console disconnected during event replay");
                return;
            }

            replayed += 1;
        }

        if count < REPLAY_BATCH_SIZE as usize {
            break;
        }
    }

    tracing::info!(replayed, "console event stream connected");
    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
    heartbeat.tick().await;

    loop {
        tokio::select! {
            message = stream.next() => {
                match message {
                    Some(Ok(AggregatedMessage::Ping(bytes))) => {
                        if session.pong(&bytes).await.is_err() {
                            tracing::debug!("console disconnected while sending pong");
                            return;
                        }
                    }
                    Some(Ok(AggregatedMessage::Pong(_))) => {}
                    Some(Ok(AggregatedMessage::Close(reason))) => {
                        tracing::debug!(?reason, "console requested connection close");
                        return;
                    }
                    Some(Err(error)) => {
                        tracing::warn!(%error, "console WebSocket stream failed");
                        return;
                    }
                    None => {
                        tracing::debug!("console WebSocket stream ended");
                        return;
                    }
                    Some(Ok(AggregatedMessage::Text(_) | AggregatedMessage::Binary(_))) => {
                        tracing::warn!("console attempted to write to its read-only event stream");
                        close(session, CloseCode::Policy, "console stream is read only").await;
                        return;
                    }
                }
            }
            notification = events.dequeue() => {
                let (delivery, event) = match notification {
                    Some(Ok(delivery)) => delivery,
                    Some(Err(error)) => {
                        tracing::error!(%error, "failed to consume console AMQP event");
                        close(session, CloseCode::Error, "event subscription failed").await;
                        return;
                    }
                    None => {
                        tracing::warn!("console AMQP subscription ended");
                        close(session, CloseCode::Again, "event subscription ended").await;
                        return;
                    }
                };

                if event.tenant_id == tenant_id && sent.insert(event.id) {
                    if let Err(error) = emit(&mut session, &event).await {
                        tracing::debug!(%error, event_id = %event.id, trace_id = %event.trace_id, "console disconnected during live event");
                        return;
                    }

                    tracing::debug!(
                        event_key = %event.key,
                        event_id = %event.id,
                        trace_id = %event.trace_id,
                        "emitted live console event"
                    );
                }

                if let Err(error) = delivery
                    .ack(amqp::lapin::options::BasicAckOptions::default())
                    .await
                {
                    tracing::error!(
                        %error,
                        event_key = %event.key,
                        event_id = %event.id,
                        trace_id = %event.trace_id,
                        "failed to acknowledge console AMQP event"
                    );
                    close(session, CloseCode::Error, "event acknowledgement failed").await;
                    return;
                }
            }
            _ = heartbeat.tick() => {
                if session.ping(b"neuron").await.is_err() {
                    tracing::debug!("console disconnected while sending heartbeat");
                    return;
                }
            }
        }
    }
}

async fn emit(session: &mut actix_ws::Session, event: &types::events::Event) -> error::Result<()> {
    session.text(serde_json::to_string(event)?).await.map_err(error::http)
}

async fn close(session: actix_ws::Session, code: CloseCode, description: &str) {
    let _ = session
        .close(Some(CloseReason {
            code,
            description: Some(description.to_string()),
        }))
        .await;
}
