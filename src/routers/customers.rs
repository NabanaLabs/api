use axum::extract::rejection::JsonRejection;
use axum::extract::Query;
use axum::routing::{get, patch};
use axum::{BoxError, Json};
use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::post};
use crate::controllers::customer::{create_customer_record, fetch_customer_record_by_id, update_name, update_password};
use crate::controllers::email::{add_email, verify_email};
use crate::types::incoming_requests::{CustomerAddEmail, CustomerUpdateName, CustomerUpdatePassword, FetchCustomerByID};
use crate::types::state::AppState;

use std::{sync::Arc, time::Duration};

use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

// /api/customers
pub async fn get_customers_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    return Router::new()
        // fetch customer by id
        .route(
            "",
            get({
                let app_state = Arc::clone(&app_state);
                move |(headers, query): (HeaderMap, Query<FetchCustomerByID>)| fetch_customer_record_by_id(headers, query, app_state)
            })
        )
        // create customer
        .route(
            "",
            post({
                let app_state = Arc::clone(&app_state);
                move |payload| create_customer_record(payload, app_state)
            }),
        )
        // update name
        .route(
            "/name",
            patch({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload): (HeaderMap, Result<Json<CustomerUpdateName>, JsonRejection>)| {
                    update_name(headers, payload, app_state)
                }
            }),
        )
        // update password
        .route(
            "/password",
            patch({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload): (HeaderMap, Result<Json<CustomerUpdatePassword>, JsonRejection>)| {
                    update_password(headers, payload, app_state)
                }
            }),
        )
        // add email
        .route(
            "/email",
            post({
                let app_state = Arc::clone(&app_state);
                move |(headers, payload): (HeaderMap, Result<Json<CustomerAddEmail>, JsonRejection>)| {
                    add_email(headers, payload, app_state)
                }
            }),
        )
        // verify email
        .route(
            "/email",
            patch({
                let app_state = Arc::clone(&app_state);
                move |query_params| {
                   verify_email(query_params, app_state)
                }
            }),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(32))
                .layer(RateLimitLayer::new(15, Duration::from_secs(60))),
        );
}