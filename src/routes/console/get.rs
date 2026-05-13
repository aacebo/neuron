use actix_web::{get, web};
use askama::Template;

#[derive(Template)]
#[template(path = "console/page.html")]
struct ConsolePage;

#[get("/console")]
pub async fn get() -> Result<web::Html, actix_web::Error> {
    Ok(web::Html::new(
        ConsolePage
            .render()
            .map_err(actix_web::error::ErrorInternalServerError)?,
    ))
}
