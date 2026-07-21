use actix_web::{Error, HttpRequest, HttpResponse, error, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;

use crate::RequestContext;

#[get("/agents/connect")]
pub async fn connect(ctx: RequestContext, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let agent_id = uuid::Uuid::from_slice(
        req.headers()
            .get("X-Agent-Id")
            .ok_or(error::ErrorUnauthorized("unauthorized"))?
            .as_bytes(),
    )
    .map_err(error::ErrorUnauthorized)?;

    let Some(actor) = ctx.storage().actors().get(agent_id).await.map_err(error::ErrorUnauthorized)? else {
        return Err(error::ErrorUnauthorized("actor not found"));
    };

    let Some(_agent) = &actor.agent else {
        return Err(error::ErrorUnauthorized("invalid actor"));
    };

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations().max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                _ => {}
            }
        }
    });

    Ok(res)
}
