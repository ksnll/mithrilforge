use std::marker::PhantomData;

use axum::extract::{FromRef, FromRequestParts, Query};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use http::request::Parts;
use serde::Deserialize;

use crate::domain::website::ports::WebsiteService;

use super::{AppState, handlers::ApiError};

pub struct Jwt<WS> {
    _marker: PhantomData<WS>,
    pub user_id: String,
}

pub struct QueryJwt<WS> {
    _marker: PhantomData<WS>,
    pub user_id: String,
}

#[derive(Deserialize, Debug)]
struct Claims {}

impl<S, WS> FromRequestParts<S> for Jwt<WS>
where
    WS: WebsiteService,
    AppState<WS>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| ApiError::Unauthorized("token not found".to_owned()))?;
        let state = AppState::from_ref(state);
        let claims = state
            .jwt_verifier
            .verify::<Claims>(bearer.token())
            .await
            .map_err(|e| ApiError::Unauthorized(e.to_string()))?;
        Ok(Jwt::<WS> {
            _marker: PhantomData,
            user_id: claims
                .claims()
                .sub
                .clone()
                .ok_or(ApiError::Unauthorized("failed to read sub".to_string()))?,
        })
    }
}

#[derive(Deserialize, Debug)]
struct QueryToken {
    token: String,
}

impl<S, WS> FromRequestParts<S> for QueryJwt<WS>
where
    WS: WebsiteService,
    AppState<WS>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<QueryToken>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized("token not found".to_owned()))?;
        let state = AppState::from_ref(state);
        let claims = state
            .jwt_verifier
            .verify::<Claims>(&query.token)
            .await
            .map_err(|e| ApiError::Unauthorized(e.to_string()))?;
        Ok(QueryJwt::<WS> {
            _marker: PhantomData,
            user_id: claims
                .claims()
                .sub
                .clone()
                .ok_or(ApiError::Unauthorized("failed to read sub".to_string()))?,
        })
    }
}
