use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use error::Result;
use futures_util::StreamExt;

use crate::RequestContext;

#[get("/agents/connect")]
pub async fn connect(ctx: RequestContext, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let agent_id = req
        .headers()
        .get("X-Agent-Id")
        .ok_or_else(|| error::http("unauthorized"))?
        .to_str()
        .map_err(error::parse)?
        .parse::<uuid::Uuid>()
        .map_err(error::parse)?;

    let Some(mut actor) = ctx.storage().actors().get_by_id(agent_id).await? else {
        return Err(error::http("unauthorized"));
    };

    let Some(agent) = &mut actor.agent else {
        return Err(error::http("unauthorized"));
    };

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations().max_continuation_size(2_usize.pow(20));
    let actor_id = actor.id;

    agent.instances += 1;
    agent.status = types::actors::AgentStatus::Online;
    actor = ctx.storage().actors().update(actor).await?;
    ctx.enqueue(actor.tenant_id, "actor.update", actor).await?;

    rt::spawn(async move {
        while let Some(message) = stream.next().await {
            if let Ok(message) = message {
                match message {
                    AggregatedMessage::Binary(bytes) => session.binary(bytes).await.unwrap(),
                    AggregatedMessage::Close(_reason) => break,
                    _ => {}
                }
            }
        }

        let Ok(Some(mut actor)) = ctx.storage().actors().get_by_id(actor_id).await else {
            return;
        };

        let Some(agent) = &mut actor.agent else {
            return;
        };

        agent.instances -= 1;
        agent.status = if agent.instances == 0 {
            types::actors::AgentStatus::Offline
        } else {
            types::actors::AgentStatus::Online
        };

        actor = ctx.storage().actors().update(actor).await.unwrap();
        let _ = ctx.enqueue(actor.tenant_id, "actor.update", actor).await;
    });

    Ok(res)
}
