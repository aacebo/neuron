use std::future::{Ready, ready};
use std::pin::Pin;

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev, web};

use crate::RequestContext;

const AGENT_ID_HEADER: &str = "X-Agent-Id";
const AGENT_SECRET_HEADER: &str = "X-Agent-Secret";
const INVALID_CREDENTIALS: &str = "invalid agent credentials";

#[derive(Clone, PartialEq, Eq, serde::Deserialize)]
struct Credentials {
    agent_id: uuid::Uuid,
    secret: String,
}

impl FromRequest for Credentials {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let agent_id = req.headers().get(AGENT_ID_HEADER);
        let secret = req.headers().get(AGENT_SECRET_HEADER);
        let credentials = match (agent_id, secret) {
            (Some(agent_id), Some(secret)) => {
                let agent_id = agent_id
                    .to_str()
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| error::unauthorized(INVALID_CREDENTIALS));
                let secret = secret
                    .to_str()
                    .map(str::to_owned)
                    .map_err(|_| error::unauthorized(INVALID_CREDENTIALS));

                agent_id.and_then(|agent_id| secret.map(|secret| Self { agent_id, secret }))
            }
            (None, None) => web::Query::<Self>::from_query(req.query_string())
                .map(web::Query::into_inner)
                .map_err(|_| error::unauthorized(INVALID_CREDENTIALS)),
            _ => Err(error::unauthorized(INVALID_CREDENTIALS)),
        };

        ready(credentials)
    }
}

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

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let credentials = Credentials::from_request(req, payload);
        let ctx = req
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("RequestContext not found in request extensions");

        Box::pin(async move {
            let Credentials { agent_id, secret } = match credentials.await {
                Ok(credentials) => credentials,
                Err(error) => {
                    tracing::warn!("agent authentication rejected");
                    return Err(error);
                }
            };

            let stored_secret = match ctx.storage().actors().get_secret(agent_id).await {
                Ok(Some(secret)) => secret,
                Ok(None) => {
                    tracing::warn!(%agent_id, "agent authentication rejected");
                    return Err(error::unauthorized(INVALID_CREDENTIALS));
                }
                Err(error) => {
                    tracing::error!(%error, %agent_id, "failed to load agent credentials");
                    return Err(error);
                }
            };

            if stored_secret != secret {
                tracing::warn!(%agent_id, "agent authentication rejected");
                return Err(error::unauthorized(INVALID_CREDENTIALS));
            }

            let actor = match ctx.storage().actors().get_by_id(agent_id).await {
                Ok(Some(actor)) => actor,
                Ok(None) => {
                    tracing::warn!(%agent_id, "agent authentication rejected");
                    return Err(error::unauthorized(INVALID_CREDENTIALS));
                }
                Err(error) => {
                    tracing::error!(%error, %agent_id, "failed to load authenticated agent");
                    return Err(error);
                }
            };

            if actor.agent.is_none() {
                tracing::warn!(%agent_id, "agent authentication rejected");
                return Err(error::unauthorized(INVALID_CREDENTIALS));
            }

            tracing::debug!(%agent_id, tenant_id = %actor.tenant_id, "agent authenticated");
            Ok(Self(actor))
        })
    }
}
