use std::time::Duration;

use actix_web::{HttpResponse, get, web};
use bytes::Bytes;
use futures_util::stream;
use storage::types::JobStatus;

use crate::RequestContext;

#[get("/messages/{id}/events")]
pub async fn events(ctx: RequestContext, path: web::Path<uuid::Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let pool = ctx.pool().clone();

    let s = stream::unfold((false, None::<JobStatus>), move |(done, last_status)| {
        let pool = pool.clone();
        async move {
            if done {
                return None;
            }

            loop {
                actix_web::rt::time::sleep(Duration::from_millis(500)).await;
                let storage = storage::Storage::new(&pool);

                let jobs = match storage.jobs().get_by_message(id).await {
                    Ok(j) => j,
                    Err(_) => continue,
                };

                let current_status = jobs.first().map(|j| j.status);

                // only emit when status changes
                if current_status == last_status {
                    continue;
                }

                let message = match storage.messages().get(id).await {
                    Ok(Some(m)) => m,
                    _ => continue,
                };
                let annotations = match storage.annotations().get_by_message(id).await {
                    Ok(a) => a,
                    Err(_) => continue,
                };
                let artifacts = match storage.artifacts().get_by_message(id).await {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                let payload = serde_json::json!({
                    "id": message.id,
                    "text": message.text,
                    "source": message.source,
                    "created_at": message.created_at,
                    "updated_at": message.updated_at,
                    "annotations": annotations,
                    "artifacts": artifacts,
                });

                let is_terminal = matches!(
                    current_status,
                    Some(JobStatus::Success)
                        | Some(JobStatus::Failure)
                        | Some(JobStatus::Cancelled)
                );

                let mut frame = format!(
                    "data: {}\n\n",
                    serde_json::to_string(&payload).unwrap_or_default()
                );

                if is_terminal {
                    frame.push_str("event: done\ndata: \n\n");
                }

                return Some((
                    Ok::<Bytes, actix_web::Error>(Bytes::from(frame)),
                    (is_terminal, current_status),
                ));
            }
        }
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(s)
}
