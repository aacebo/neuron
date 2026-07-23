use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest, dev, web};

use crate::RequestContext;

#[derive(Debug)]
pub struct Agent(types::actors::Actor);

impl std::ops::Deref for Agent {
    type Target = types::actors::Actor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Agent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRequest for Agent {
    type Error = error::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let ctx = req.app_data::<web::Data<RequestContext>>().unwrap().clone().into_inner();
        let agent_id = req.headers().get("X-Agent-Id").cloned();
        let secret = req.headers().get("X-Agent-Secret").cloned();

        Box::pin(async move {
            let (agent_id, secret) = match (agent_id, secret) {
                (Some(agent_id), Some(secret)) => agent_id
                    .to_str()
                    .map_err(error::parse)
                    .and_then(|value| value.parse::<uuid::Uuid>().map_err(error::parse))
                    .and_then(|agent_id| {
                        secret
                            .to_str()
                            .map(|secret| (agent_id, secret.to_string()))
                            .map_err(error::parse)
                    }),
                _ => Err(error::unauthorized("both X-Agent-Id and X-Agent-Secret are required")),
            }?;

            let Some(stored_secret) = ctx.storage().actors().get_secret(agent_id).await? else {
                return Err(error::unauthorized("invalid agent credentials"));
            };

            if stored_secret != secret {
                return Err(error::unauthorized("invalid agent credentials"));
            }

            let Some(actor) = ctx.storage().actors().get_by_id(agent_id).await? else {
                return Err(error::unauthorized("invalid agent credentials"));
            };

            if actor.agent.is_none() {
                return Err(error::unauthorized("invalid agent credentials"));
            }

            Ok(Self(actor))
        })
    }
}
