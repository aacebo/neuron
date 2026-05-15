use actix_web::Error;
use actix_web::{HttpResponse, post};
use amqp::{Action, Event, Key};
use storage::types::Message;

use crate::RequestContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CreateMessage {
    pub text: String,
}

#[post("/chats/{chat}/messages")]
pub async fn create(
    ctx: RequestContext,
    body: actix_web::web::Json<CreateMessage>,
) -> Result<HttpResponse, Error> {
    let mut message = Message::new(&body.text);

    message = ctx
        .storage()
        .messages()
        .create(&message)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    ctx.amqp()
        .produce()
        .enqueue(Event::new(
            Key::new("message", Action::Create),
            message.clone(),
        ))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(message))
}
