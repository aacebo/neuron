mod connect;

use actix_web::{HttpResponse, get, web};
use askama::Template;

use crate::RequestContext;

#[derive(Clone, Template, serde::Serialize, serde::Deserialize)]
#[template(path = "console/index.html")]
struct ConsoleTemplate {
    tenant_id: uuid::Uuid,
    high_water_cursor: Option<storage::EventCursor>,
    reducer_version: u32,
}

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(web::scope("/console").service(page).service(connect::connect));
}

#[get("")]
async fn page(ctx: RequestContext) -> error::Result<HttpResponse> {
    let tenant_id = ctx.console().tenant_id.unwrap();
    let template = ConsoleTemplate {
        tenant_id,
        high_water_cursor: ctx.storage().events().latest_cursor(tenant_id).await?,
        reducer_version: 2,
    };

    let body = template.render().map_err(error::http)?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .content_type("text/html; charset=utf-8")
        .body(body))
}
