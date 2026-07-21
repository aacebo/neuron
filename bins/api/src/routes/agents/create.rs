use ::error::IntoError;
use actix_web::{HttpResponse, post, web};

use crate::{RequestContext, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CreateAgent {
    pub external_id: Option<String>,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub skills: Vec<types::actors::Skill>,
}

#[post("/tenants/{tenant_id}/agents")]
pub async fn create(ctx: RequestContext, tenant_id: web::Path<uuid::Uuid>, body: web::Json<CreateAgent>) -> Result<HttpResponse> {
    let body = body.into_inner();
    let actor = ctx
        .storage()
        .actors()
        .create(types::actors::Actor {
            id: uuid::Uuid::new_v4(),
            external_id: body.external_id,
            tenant_id: tenant_id.into_inner(),
            role: types::actors::Role::Agent,
            name: body.name,
            display_name: body.display_name,
            agent: Some(types::actors::Agent {
                status: types::actors::AgentStatus::Offline,
                description: body.description,
                skills: body.skills,
            }),
            metadata: Default::default(),
            embedding: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await
        .map_err(IntoError::into_error)?;

    let res = HttpResponse::Created().json(&actor);
    ctx.enqueue("actor.create", actor).await?;

    Ok(res)
}
