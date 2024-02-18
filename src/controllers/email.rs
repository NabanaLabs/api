use std::sync::Arc;

use axum::{extract::{rejection::JsonRejection, Query}, http::{HeaderMap, StatusCode}, Json};
use chrono::Utc;
use mongodb::bson::doc;
use redis::{Commands, RedisError};

use crate::{email::brevo_api::send_verification_email, storage::mongo::{build_customer_filter, find_customer, update_customer}, types::{customer::{Email, GenericResponse}, email::SendEmailData, incoming_requests::{CustomerAddEmail, VerifyEmailQueryParams}, state::AppState}, utilities::helpers::{bad_request, internal_server_error, ok, payload_analyzer, random_string, unauthorized, valid_email}};

use super::identity::{get_user_session_from_req, SessionScopes};

pub async fn add_email(
    headers: HeaderMap,
    payload_result: Result<Json<CustomerAddEmail>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = match get_user_session_from_req(&headers, &state.redis_connection).await {
        Ok(customer_id) => customer_id,
        Err(_) => return Err(unauthorized("", None)),
    };

    if !(session_data.scopes.contains(&SessionScopes::TotalAccess) && session_data.scopes.contains(&SessionScopes::UpdateEmailAddresses))
    {
        return Err(unauthorized("", None));
    }

    let payload = payload_analyzer(payload_result)?;

    let filter = build_customer_filter(session_data.customer_id.as_str(), "").await;
    let customer = find_customer(&state.mongo_db, filter).await?;

    let mut emails = customer.emails;
    if emails.len() >= 5 {
        return Err(bad_request("email.limit.reached", None));
    }

    let email = payload.email.to_lowercase();
    match valid_email(&email).await {
        Ok(_) => (),
        Err(_) => return Err(bad_request("invalid.email", None)),
    };

    for registered_email in emails.iter() {
        if registered_email.address == email {
            return Err(bad_request("email.already.registered", None));
        }
    }

    let filter = build_customer_filter("", email.as_str()).await;
    let customer_against = find_customer(&state.mongo_db, filter).await;

    match customer_against {
        Ok(customer_against) => {
            if customer_against.id != customer.id {
                return Err(bad_request("email.already.registered", None));
            }
    
            return Err(bad_request("email.already.registered.by.you", None));
        },
        Err(_) => (),
    }

    emails.push(Email {
        address: email.clone(),
        verified: false,
        main: false,
    });

    let bson_emails = emails
        .iter()
        .map(|email| {
            doc! {
                "address": &email.address,
                "verified": &email.verified,
                "main": &email.main,
            }
        })
        .collect::<Vec<_>>();

    let current_datetime = Utc::now();
    let iso8601_string = current_datetime.to_rfc3339();

    let filter = build_customer_filter(session_data.customer_id.as_str(), "").await;
    let update = doc! {"$set": {
            "emails": &bson_emails,
            "updated_at": iso8601_string,
        }
    };

    match update_customer(&state.mongo_db, filter, update).await {
        Ok(_) => {
            let api_key = match std::env::var("BREVO_CUSTOMERS_WEBFLOW_API_KEY") {
                Ok(api_key) => api_key,
                Err(_) => {
                    return Err(internal_server_error("", None));
                }
            };
            
            let _ = new_email_verification(
                &state,
                api_key,
                email,
                customer.name,
            ).await;
        },
        Err(_) => return Err(internal_server_error("database.error", None)),
    }

    return Ok(ok("email.added", None));
}

pub async fn verify_email(
    Query(params): Query<VerifyEmailQueryParams>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let token = match params.token {
        Some(token) => token,
        None => return Err(bad_request("invalid.token", None))
    };

    let mut redis_conn = match state.redis_connection.get_connection() {
        Ok(redis_conn) => redis_conn,
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    let customer_email_address: String = match redis_conn.get(token.clone()) {
        Ok(customer_email_address) => customer_email_address,
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    if customer_email_address.is_empty() {
        return Err(bad_request("invalid.token", None));
    }

    let filter = doc! {
        "emails.address": customer_email_address,
    };

    let update = doc! {
        "$set": {
            "emails.$.verified": true,
        }
    };

    match update_customer(&state.mongo_db, filter, update).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("database.error", None)),
    };

    let result: Result<bool, RedisError> = redis_conn.del(token.clone());
    match result {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    return Ok(ok("email.verified", None));
}

pub async fn new_email_verification(
    state: &Arc<AppState>,
    api_key: String,
    customer_email: String,
    customer_name: String,
) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let new_token = random_string(30).await;
    let mut redis_conn = match state.redis_connection.get_connection() {
        Ok(redis_conn) => redis_conn,
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    let result: Result<bool, RedisError> = redis_conn.set_ex(
        new_token.clone(),
        &customer_email,
        state.api_tokens_expiration_time.try_into().unwrap_or(86000),
    );

    match result {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    let greetings_title = format!("Welcome to Test App {}", customer_name);
    let verification_link = format!("{}?token={}", state.google_auth.redirect_url, new_token);
    let send_email_data = SendEmailData {
        api_key,
        subject: "Verify Your New Email Address".to_string(),
        template_id: state.email_provider_settings.email_verification_template_id,
        customer_email: customer_email,
        customer_name: customer_name.clone(),
        verification_link,
        greetings_title,
        sender_email: state.master_email_entity.email.clone(),
        sender_name: state.master_email_entity.name.clone(),
    };

    match send_verification_email(send_email_data).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("email.error", None)),
    };

    return Ok(());
}
