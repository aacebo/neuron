use actix_web::{get, web};
use askama::Template;

#[derive(Template)]
#[template(path = "console/page.html")]
struct ConsolePage {
    initial_message_id_json: String,
}

impl ConsolePage {
    fn new(initial_message_id: Option<uuid::Uuid>) -> Self {
        Self {
            initial_message_id_json: initial_message_id
                .map(|id| format!("\"{id}\""))
                .unwrap_or_else(|| "null".to_string()),
        }
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
pub async fn get_run(path: web::Path<uuid::Uuid>) -> Result<web::Html, actix_web::Error> {
    Ok(web::Html::new(
        ConsolePage::new(Some(path.into_inner()))
            .render()
            .map_err(actix_web::error::ErrorInternalServerError)?,
    ))
}
