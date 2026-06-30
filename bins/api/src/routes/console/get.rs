use actix_web::{get, web};
use askama::Template;

use crate::RequestContext;
use crate::views::MessageView;

#[derive(Template)]
#[template(path = "console/page.html")]
struct ConsolePage {
    message: Option<MessageView>,
}

impl ConsolePage {
    fn new(message: Option<MessageView>) -> Self {
        Self { message }
    }
}

#[get("/console")]
pub async fn get() -> Result<web::Html, actix_web::Error> {
    Ok(web::Html::new(
        ConsolePage::new(None)
            .render()
            .map_err(actix_web::error::ErrorInternalServerError)?,
    ))
}

#[get("/console/{message_id}")]
pub async fn get_run(
    ctx: RequestContext,
    path: web::Path<uuid::Uuid>,
) -> Result<web::Html, actix_web::Error> {
    let message_id = path.into_inner();
    let message = MessageView::get(&ctx.storage(), message_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("message not found"))?;

    Ok(web::Html::new(
        ConsolePage::new(Some(message))
            .render()
            .map_err(actix_web::error::ErrorInternalServerError)?,
    ))
}
