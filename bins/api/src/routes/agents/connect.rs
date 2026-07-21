use ::error::IntoError;
use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;

use crate::{Error, RequestContext, Result};

#[get("/agents/connect")]
pub async fn connect(ctx: RequestContext, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let agent_id = uuid::Uuid::from_slice(
        req.headers()
            .get("X-Agent-Id")
            .ok_or_else(|| ::error::unauthorized("missing X-Agent-Id header"))?
            .as_bytes(),
    )
    .map_err(::error::unauthorized)?;

    let Some(actor) = ctx
        .storage()
        .actors()
        .get_by_id(agent_id)
        .await
        .map_err(IntoError::into_error)?
    else {
        return Err(::error::unauthorized("actor not found").into());
    };

    let Some(_agent) = &actor.agent else {
        return Err(::error::unauthorized("invalid actor").into());
    };

    let (res, mut session, stream) = actix_ws::handle(&req, stream).map_err(Error::request)?;
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
