use axum::routing::{delete, get, patch};
use axum::BoxError;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{Router, routing::post};
use crate::controllers::org::{create_model_org, create_org, create_router_org, delete_model_org, delete_org, edit_model_org, edit_org, edit_router_org, edit_router_prompt_classification_org, edit_router_sentence_matching_org, edit_router_single_model_org, get_models, get_org, get_routers};
use crate::types::state::AppState;
use std::{sync::Arc, time::Duration};

use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

// /api/org
pub async fn get_org_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    return Router::new()
        .route(
            // fetch org
            "/",
            get({
                let app_state = Arc::clone(&app_state);
                move |headers| get_org(headers, app_state) 
            }),
        )
        .route(
            // create org
            "/",
            post({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| create_org(headers, payload, app_state)
            }),
        )
        .route(
            // delete org
            "/",
            delete({
                let app_state = Arc::clone(&app_state);
                move |headers| delete_org(headers, app_state)
            }),
        )
        .route(
            // edit org
            "/",
            patch({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| edit_org(headers, payload, app_state)
            }),
        )
        .route(
            // fetch model
            "/models",
            get({
                let app_state = Arc::clone(&app_state);
                move |headers| get_models(headers, app_state)
            }),
        )
        .route(
            // create model
            "/models",
            post({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| create_model_org(headers, payload, app_state)
            }),
        )
        .route(
            // delete model
            "/models",
            delete({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| delete_model_org(headers, payload, app_state)
            }),
        )
        .route(
            // edit model
            "/models",
            patch({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| edit_model_org(headers, payload, app_state)
            }),
        )
        .route(
            // fetch routers
            "/routers",
            get({
                let app_state = Arc::clone(&app_state);
                move |headers| get_routers(headers, app_state)
            }),
        )
        .route(
            // create routers
            "/routers",
            post({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload)| create_router_org(headers, payload, app_state)
            }),
        )
        .route(
            // edit routers
            "/routers", 
            patch({
            let app_state = Arc::clone(&app_state);
            move |(headers, payload)| edit_router_org(headers, payload, app_state)
        }))
        .route(
            // edit routers prompt classification model
            "/routers/single.model", 
            patch({
            let app_state = Arc::clone(&app_state);
            move |(headers, payload)| edit_router_single_model_org(headers, payload, app_state)
        }))
        .route(
            // edit routers prompt classification model
            "/routers/prompt.classification.model", 
            patch({
            let app_state = Arc::clone(&app_state);
            move |(headers, payload)| edit_router_prompt_classification_org(headers, payload, app_state)
        }))
        .route(
            // edit routers
            "/routers/sentence.matching", 
            patch({
            let app_state = Arc::clone(&app_state);
            move |(headers, payload)| edit_router_sentence_matching_org(headers, payload, app_state)
        }))
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
