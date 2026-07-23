use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest, dev, web};

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> FromRequest for Json<T>
where
    T: serde::de::DeserializeOwned + serde_valid::Validate + 'static,
{
    type Error = error::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let future = web::Json::<T>::from_request(req, payload);

        Box::pin(async move {
            let body = future.await?.into_inner();
            body.validate()?;
            Ok(Self(body))
        })
    }
}
