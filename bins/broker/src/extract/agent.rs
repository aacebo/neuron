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

#[cfg(test)]
mod tests {
    use actix_web::FromRequest;
    use actix_web::test::TestRequest;

    use super::Credentials;

    #[actix_web::test]
    async fn resolves_complete_header_credentials() {
        let agent_id = uuid::Uuid::new_v4();
        let (request, mut payload) = TestRequest::default()
            .insert_header(("X-Agent-Id", agent_id.to_string()))
            .insert_header(("X-Agent-Secret", "header-secret"))
            .to_http_parts();

        let credentials = Credentials::from_request(&request, &mut payload).await.unwrap();
        assert_eq!(credentials.agent_id, agent_id);
        assert_eq!(credentials.secret, "header-secret");
    }

    #[actix_web::test]
    async fn falls_back_to_query_credentials_when_headers_are_absent() {
        let agent_id = uuid::Uuid::new_v4();
        let (request, mut payload) =
            TestRequest::with_uri(&format!("/agents/connect?agent_id={agent_id}&secret=query%20secret%26value")).to_http_parts();

        let credentials = Credentials::from_request(&request, &mut payload).await.unwrap();
        assert_eq!(credentials.agent_id, agent_id);
        assert_eq!(credentials.secret, "query secret&value");
    }

    #[actix_web::test]
    async fn complete_headers_take_precedence_over_query_credentials() {
        let header_id = uuid::Uuid::new_v4();
        let query_id = uuid::Uuid::new_v4();
        let (request, mut payload) = TestRequest::with_uri(&format!("/agents/connect?agent_id={query_id}&secret=query-secret"))
            .insert_header(("X-Agent-Id", header_id.to_string()))
            .insert_header(("X-Agent-Secret", "header-secret"))
            .to_http_parts();

        let credentials = Credentials::from_request(&request, &mut payload).await.unwrap();
        assert_eq!(credentials.agent_id, header_id);
        assert_eq!(credentials.secret, "header-secret");
    }

    #[actix_web::test]
    async fn partial_headers_do_not_fall_back_to_query_credentials() {
        let query_id = uuid::Uuid::new_v4();
        let (request, mut payload) = TestRequest::with_uri(&format!("/agents/connect?agent_id={query_id}&secret=query-secret"))
            .insert_header(("X-Agent-Id", uuid::Uuid::new_v4().to_string()))
            .to_http_parts();

        let Err(error) = Credentials::from_request(&request, &mut payload).await else {
            panic!("partial headers must be rejected");
        };
        assert_eq!(error.name(), "unauthorized");
        assert_eq!(error.message(), "invalid agent credentials");
    }

    #[actix_web::test]
    async fn malformed_or_missing_credentials_are_unauthorized() {
        for uri in [
            "/agents/connect",
            "/agents/connect?agent_id=invalid&secret=value",
            "/agents/connect?secret=value",
        ] {
            let (request, mut payload) = TestRequest::with_uri(uri).to_http_parts();
            let Err(error) = Credentials::from_request(&request, &mut payload).await else {
                panic!("malformed or missing credentials must be rejected");
            };
            assert_eq!(error.name(), "unauthorized");
            assert_eq!(error.message(), "invalid agent credentials");
        }
    }
}
