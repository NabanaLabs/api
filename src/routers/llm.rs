use axum::routing::get;
use axum::BoxError;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{Router, routing::post};
use crate::controllers::llm::{get_models_list, process_prompt};
use crate::types::state::AppState;
use std::{sync::Arc, time::Duration};

use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

// /api/core
pub async fn get_llm_routers_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    return Router::new()
        .route(
            // suggest and model and cache it
            // if cache is not empty, return the cached response
            "/get/models",
            get({
                move || get_models_list()
            }),
        )
        .route(
            // suggest and model and cache it
            // if cache is not empty, return the cached response
            "/prompt",
            post({
                let app_state = Arc::clone(&app_state);
                move |payload| process_prompt(payload, app_state)
            }),
        )
        /*.route(
            // check if prompt is in cache
            "/prompt/cache",
            get({
                let app_state = Arc::clone(&app_state);
                move |payload| process_prompt(payload, app_state)
            }),
        )
        .route(
            // add prompt-response to cache
            "/prompt/cache",
            post({
                let app_state = Arc::clone(&app_state);
                move |payload| process_prompt(payload, app_state)
            }),
        ) */
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(256))
                .layer(RateLimitLayer::new(256, Duration::from_secs(60))),
        );
}