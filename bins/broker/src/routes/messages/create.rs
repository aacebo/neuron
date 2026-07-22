use actix_web::{HttpResponse, post};
use error::Result;
use serde_valid::Validate;

use crate::{RequestContext, extract};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
struct Request {
    pub tenant_id: uuid::Uuid,
    #[validate(min_items = 1)]
    pub content: Vec<types::data::Content>,
    #[serde(default)]
    pub metadata: types::data::Metadata,
    pub from: FromUser,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
struct FromUser {
    pub id: String,
    pub name: String,
}

#[post("/messages")]
pub async fn create(ctx: RequestContext, body: extract::Json<Request>) -> Result<HttpResponse> {
    let body = body.into_inner();
    let _from = match ctx
        .storage()
        .actors()
        .get_by_external_id(body.tenant_id, body.from.id.clone())
        .await?
    {
        Some(actor) => actor,
        None => {
            let actor = ctx
                .storage()
                .actors()
                .create(types::actors::Actor {
                    id: uuid::Uuid::new_v4(),
                    external_id: Some(body.from.id.clone()),
                    tenant_id: body.tenant_id,
                    role: types::actors::Role::User,
                    name: body.from.name,
                    agent: None,
                    metadata: body.metadata,
                    embedding: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .await?;

            ctx.enqueue("actor.create", actor.clone()).await?;
            actor
        }
    };

    // 1. create/update actor.
    // 2. create embedding of message content ??.
    // 3. search for agents using said embedding.
    // 4. create a new chat with the relevant agents and the from user.
    // 5. on message create, generate message summary/embedding/annotations/artifacts

    Ok(HttpResponse::Ok().finish())
}
