use actix_web::{Error, HttpResponse, get, web};

use crate::RequestContext;

#[get("/messages/{id}")]
pub async fn get(ctx: RequestContext, path: web::Path<uuid::Uuid>) -> Result<HttpResponse, Error> {
    let id = path.into_inner();

    let message = ctx
        .storage()
        .messages()
        .get(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("message not found"))?;

    let annotations = ctx
        .storage()
        .annotations()
        .get_by_message(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let artifacts = ctx
        .storage()
        .artifacts()
        .get_by_message(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": message.id,
        "text": message.text,
        "source": message.source,
        "created_at": message.created_at,
        "updated_at": message.updated_at,
        "annotations": annotations,
        "artifacts": artifacts,
    })))
}
