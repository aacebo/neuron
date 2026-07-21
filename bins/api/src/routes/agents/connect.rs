use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use error::Result;
use futures_util::StreamExt;

use crate::RequestContext;

#[get("/agents/connect")]
pub async fn connect(ctx: RequestContext, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let agent_id = uuid::Uuid::from_slice(
        req.headers()
            .get("X-Agent-Id")
            .ok_or_else(|| error::http("unauthorized"))?
            .as_bytes(),
    )
    .map_err(error::parse)?;

    let Some(actor) = ctx.storage().actors().get_by_id(agent_id).await? else {
        return Err(error::http("unauthorized"));
    };

    let Some(_agent) = &actor.agent else {
        return Err(error::http("unauthorized"));
    };

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations().max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Ok(AggregatedMessage::Binary(bin)) = msg {
                session.binary(bin).await.unwrap();
            }
        }
    });

    Ok(res)
}
