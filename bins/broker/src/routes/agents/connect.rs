use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::{AggregatedMessage, CloseCode, CloseReason};
use futures_util::StreamExt;
use serde_valid::Validate;

use crate::{RequestContext, extract};

const MAX_MESSAGE_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Command {
    MessageSend {
        #[serde(default)]
        trace_id: Option<uuid::Uuid>,
        #[serde(default)]
        chat_id: Option<uuid::Uuid>,
        #[serde(default)]
        subject: Option<String>,
        content: types::data::Contents,
        #[serde(default)]
        metadata: types::data::Metadata,
    },
}

#[get("/agents/connect")]
pub async fn connect(
    ctx: RequestContext,
    req: HttpRequest,
    stream: web::Payload,
    actor: extract::Agent,
) -> error::Result<HttpResponse> {
    let (response, session, stream) = actix_ws::handle(&req, stream)?;
    let stream = stream.aggregate_continuations().max_continuation_size(MAX_MESSAGE_SIZE);

    rt::spawn(run_session(ctx, session, stream, actor));
    Ok(response)
}

async fn run_session(
    ctx: RequestContext,
    mut session: actix_ws::Session,
    mut stream: actix_ws::AggregatedMessageStream,
    actor: extract::Agent,
) {
    let Some(actor) = ctx.storage().actors().connect(actor.id).await.ok().flatten() else {
        close(session, CloseCode::Error, "failed to update agent connection").await;
        return;
    };

    let connection_event = match ctx.enqueue(actor.tenant_id, "actor.update", actor.clone()).await {
        Ok(event) => event,
        Err(_) => {
            let _ = ctx.storage().actors().disconnect(actor.id).await;
            close(session, CloseCode::Error, "failed to persist connection event").await;
            return;
        }
    };

    if emit(&mut session, &connection_event).await.is_err() {
        disconnect(&ctx, actor.id).await;
        return;
    }

    while let Some(message) = stream.next().await {
        match message {
            Ok(AggregatedMessage::Text(text)) => {
                let Ok(command) = serde_json::from_str::<Command>(&text) else {
                    close(session.clone(), CloseCode::Invalid, "invalid command").await;
                    break;
                };

                #[allow(irrefutable_let_patterns)]
                let Command::MessageSend {
                    trace_id,
                    chat_id,
                    subject,
                    content,
                    metadata,
                } = command
                else {
                    close(session.clone(), CloseCode::Policy, "already authenticated").await;
                    break;
                };

                if content.validate().is_err() {
                    close(session.clone(), CloseCode::Invalid, "invalid message content").await;
                    break;
                }

                if let Some(chat_id) = chat_id {
                    let chat = ctx
                        .storage()
                        .chats()
                        .get_open_for_actor(chat_id, actor.tenant_id, actor.id)
                        .await;
                    if !matches!(chat, Ok(Some(_))) {
                        close(session.clone(), CloseCode::Policy, "chat is unavailable for this agent").await;
                        break;
                    }
                }

                let trace_id = trace_id.unwrap_or_else(uuid::Uuid::new_v4);
                let message = types::chats::InboundMessage {
                    tenant_id: actor.tenant_id,
                    chat_id,
                    subject,
                    content,
                    metadata,
                    sent_by: actor.clone().into(),
                };

                let event = match ctx
                    .enqueue_with_trace(actor.tenant_id, trace_id, "message.inbound", message)
                    .await
                {
                    Ok(event) => event,
                    Err(_) => {
                        close(session.clone(), CloseCode::Error, "failed to persist message").await;
                        break;
                    }
                };

                if emit(&mut session, &event).await.is_err() {
                    break;
                }
            }
            Ok(AggregatedMessage::Ping(bytes)) => {
                if session.pong(&bytes).await.is_err() {
                    break;
                }
            }
            Ok(AggregatedMessage::Pong(_)) => {}
            Ok(AggregatedMessage::Close(_)) | Err(_) => break,
            Ok(AggregatedMessage::Binary(_)) => {
                close(session.clone(), CloseCode::Unsupported, "text commands required").await;
                break;
            }
        }
    }

    disconnect(&ctx, actor.id).await;
}

async fn emit(session: &mut actix_ws::Session, event: &types::events::Event) -> error::Result<()> {
    session.text(serde_json::to_string(event)?).await.map_err(error::http)
}

async fn disconnect(ctx: &RequestContext, actor_id: uuid::Uuid) {
    let Ok(Some(actor)) = ctx.storage().actors().disconnect(actor_id).await else {
        return;
    };
    let _ = ctx.enqueue(actor.tenant_id, "actor.update", actor).await;
}

async fn close(session: actix_ws::Session, code: CloseCode, description: &str) {
    let _ = session
        .close(Some(CloseReason {
            code,
            description: Some(description.to_string()),
        }))
        .await;
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use serde_valid::Validate;

    use super::{Command, header_credentials, secrets_match};

    #[test]
    fn secret_comparison_accepts_only_exact_values() {
        assert!(secrets_match("secret", "secret"));
        assert!(!secrets_match("secret", "other!"));
        assert!(!secrets_match("secret", "short"));
    }

    #[test]
    fn parses_agent_message_command() {
        let command = serde_json::from_str::<Command>(
            r#"{
                "type":"message_send",
                "trace_id":"00000000-0000-0000-0000-000000000001",
                "content":[{"type":"text","text":"hello"}],
                "metadata":{"source":"test"}
            }"#,
        );
        assert!(matches!(command, Ok(Command::MessageSend { .. })));
    }

    #[test]
    fn parses_header_credentials_only_as_a_complete_pair() {
        let id = uuid::Uuid::new_v4();
        let request = TestRequest::default()
            .insert_header(("X-Agent-Id", id.to_string()))
            .insert_header(("X-Agent-Secret", "secret"))
            .to_http_request();
        let (actual_id, secret) = header_credentials(&request).unwrap().unwrap();
        assert_eq!(actual_id, id);
        assert_eq!(secret, "secret");

        let incomplete = TestRequest::default()
            .insert_header(("X-Agent-Id", id.to_string()))
            .to_http_request();
        assert!(header_credentials(&incomplete).unwrap().is_err());
    }

    #[test]
    fn rejects_invalid_agent_message_content() {
        let command = serde_json::from_str::<Command>(
            r#"{
                "type":"message_send",
                "content":[],
                "metadata":{}
            }"#,
        )
        .unwrap();
        let Command::MessageSend { content, .. } = command else {
            panic!("expected message_send");
        };
        assert!(content.validate().is_err());
    }
}
