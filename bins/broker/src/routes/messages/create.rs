use actix_web::{HttpResponse, post, web};
use error::Result;
use serde_valid::Validate;

use crate::RequestContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
struct Request {
    pub tenant_id: uuid::Uuid,
    pub content: Vec<types::data::Content>,
    pub metadata: types::data::Metadata,
    pub from: FromUser,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
struct FromUser {
    pub id: uuid::Uuid,
    pub external_id: Option<String>,
    pub name: String,
}

#[post("/messages")]
pub async fn create(_ctx: RequestContext, body: web::Json<Request>) -> Result<HttpResponse> {
    let _body = body.into_inner();
    Ok(HttpResponse::Ok().finish())
}
