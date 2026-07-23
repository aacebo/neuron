use std::future::{Ready, ready};
use std::sync::Arc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderMap;
use actix_web::{Error as ActixError, FromRequest, HttpMessage, HttpRequest, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use storage::Storage;
use tokio::sync::broadcast;

const REQUEST_ID_HEADER: &str = "X-Request-ID";

#[derive(Clone)]
pub struct Context {
    pool: PgPool,
    socket: amqp::Socket,
    start_time: DateTime<Utc>,
    console: crate::ConsoleConfig,
    events: Option<broadcast::Sender<types::events::Event>>,
}

impl Context {
    pub fn new(
        pool: PgPool,
        socket: amqp::Socket,
        console: crate::ConsoleConfig,
        events: Option<broadcast::Sender<types::events::Event>>,
    ) -> Self {
        Self {
            pool,
            socket,
            start_time: Utc::now(),
            console,
            events,
        }
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn storage(&self) -> Storage<'_> {
        Storage::new(&self.pool)
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn socket(&self) -> &amqp::Socket {
        &self.socket
    }

    pub fn console(&self) -> &crate::ConsoleConfig {
        &self.console
    }

    pub fn subscribe_events(&self) -> Option<broadcast::Receiver<types::events::Event>> {
        self.events.as_ref().map(broadcast::Sender::subscribe)
    }
}

#[derive(Clone)]
pub struct RequestContext {
    ctx: Arc<Context>,
    headers: HeaderMap,
    request_id: uuid::Uuid,
}

impl RequestContext {
    pub fn new(ctx: Arc<Context>, headers: HeaderMap, request_id: uuid::Uuid) -> Self {
        Self {
            ctx,
            headers,
            request_id,
        }
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn request_id(&self) -> &uuid::Uuid {
        &self.request_id
    }

    pub async fn enqueue(
        &self,
        tenant_id: uuid::Uuid,
        key: impl std::fmt::Display,
        body: impl Into<types::events::Data>,
    ) -> ::error::Result<types::events::Event> {
        self.enqueue_with_trace(tenant_id, self.request_id, key, body).await
    }

    pub async fn enqueue_with_trace(
        &self,
        tenant_id: uuid::Uuid,
        trace_id: uuid::Uuid,
        key: impl std::fmt::Display,
        body: impl Into<types::events::Data>,
    ) -> ::error::Result<types::events::Event> {
        let data = body.into();
        let event = self
            .storage()
            .events()
            .create(
                data.actor_id(),
                data.chat_id(),
                data.message_id(),
                data.task_id(),
                types::events::new(tenant_id, trace_id, key, data),
            )
            .await?;

        if let Some(events) = &self.ctx.events {
            let _ = events.send(event.clone());
        }

        self.socket.produce().enqueue(event.clone()).await?;
        Ok(event)
    }
}

impl FromRequest for RequestContext {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let ctx = req
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("RequestContext not found in request extensions");

        ready(Ok(ctx))
    }
}

impl std::ops::Deref for RequestContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context()
    }
}

pub struct RequestContextMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestContextMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Transform = RequestContextMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestContextMiddlewareService { service }))
    }
}

pub struct RequestContextMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestContextMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ctx = req
            .app_data::<web::Data<Context>>()
            .expect("Context not found in app data")
            .clone()
            .into_inner();

        let headers = req.headers().clone();
        let request_id = headers
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
            .parse()
            .unwrap_or_else(|_| uuid::Uuid::new_v4());

        let ctx = RequestContext::new(ctx, headers, request_id);
        req.extensions_mut().insert(ctx);
        self.service.call(req)
    }
}
