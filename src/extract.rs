use crate::response::ApiError;
use axum::{
    async_trait,
    extract::{ FromRequestParts, Query as AxumQuery },
    http::request::Parts,
};
use serde::de::DeserializeOwned;

/// Drop-in replacement for `axum::extract::Query` that returns our
/// `{ status, data, error }` envelope on missing/invalid query params
/// instead of axum's default plain-text 400.
pub struct Query<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        AxumQuery::<T>::from_request_parts(parts, state)
            .await
            .map(|AxumQuery(value)| Query(value))
            .map_err(|rejection| ApiError::validation(rejection.to_string(), "query"))
    }
}