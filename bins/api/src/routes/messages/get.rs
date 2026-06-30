use actix_web::{Error, HttpResponse, get, web};

use crate::RequestContext;

#[get("/messages/{id}")]
pub async fn get(ctx: RequestContext, path: web::Path<uuid::Uuid>) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let storage = ctx.storage();

    let message = storage
        .messages()
        .get(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("message not found"))?;

    let annotations = storage
        .annotations()
        .get_by_message(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let artifacts = storage
        .artifacts()
        .get_by_message(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let latest_job = storage
        .jobs()
        .get_by_message(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .into_iter()
        .next();

    let job_elapsed_ms = latest_job.as_ref().and_then(|job| {
        let started_at = job.started_at.as_ref()?;
        let ended_at = job.ended_at.as_ref()?;

        Some(
            ended_at
                .clone()
                .signed_duration_since(started_at.clone())
                .num_milliseconds()
                .max(0),
        )
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": message.id,
        "text": message.text,
        "source": message.source,
        "created_at": message.created_at,
        "updated_at": message.updated_at,
        "annotations": annotations,
        "artifacts": artifacts,
        "job_status": latest_job.as_ref().map(|job| job.status),
        "job_error": latest_job.as_ref().and_then(|job| job.error.clone()),
        "job_started_at": latest_job.as_ref().and_then(|job| job.started_at),
        "job_ended_at": latest_job.as_ref().and_then(|job| job.ended_at),
        "job_elapsed_ms": job_elapsed_ms,
    })))
}
