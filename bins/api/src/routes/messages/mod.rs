mod events;

use actix_web::{Error, HttpResponse, get, web};
pub use events::get_events;

use crate::RequestContext;
use crate::views::MessageView;

#[get("/messages/{id}")]
pub async fn get(ctx: RequestContext, path: web::Path<uuid::Uuid>) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let message = MessageView::get(&ctx.storage(), id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("message not found"))?;

    Ok(HttpResponse::Ok().json(message))
}
