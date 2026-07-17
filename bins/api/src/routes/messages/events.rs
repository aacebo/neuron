use std::time::Duration;

use actix_web::{HttpResponse, get, web};
use bytes::Bytes;
use futures_util::stream;
use storage::types::TaskStatus;

use crate::RequestContext;
use crate::views::MessageView;

#[get("/messages/{id}/events")]
pub async fn get_events(ctx: RequestContext, path: web::Path<uuid::Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let pool = ctx.pool().clone();
    let s = stream::unfold((false, None::<Vec<(uuid::Uuid, TaskStatus)>>), move |(done, last_jobs)| {
        let pool = pool.clone();
        async move {
            if done {
                return None;
            }

            loop {
                actix_web::rt::time::sleep(Duration::from_millis(500)).await;
                let storage = storage::Storage::new(&pool);

                let view = match MessageView::get(&storage, id).await {
                    Ok(Some(view)) => view,
                    Ok(None) => continue,
                    Err(_) => continue,
                };

                let current_jobs = view.tasks.iter().map(|task| (task.id, task.status)).collect::<Vec<_>>();

                // only emit when task state changes
                if last_jobs.as_ref() == Some(&current_jobs) {
                    continue;
                }

                let is_terminal = matches!(
                    view.status(),
                    Some(TaskStatus::Success) | Some(TaskStatus::Failure) | Some(TaskStatus::Cancelled)
                );

                let mut frame = format!("data: {}\n\n", serde_json::to_string(&view).unwrap_or_default());

                if is_terminal {
                    frame.push_str("event: done\ndata: \n\n");
                }

                return Some((
                    Ok::<Bytes, actix_web::Error>(Bytes::from(frame)),
                    (is_terminal, Some(current_jobs)),
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
