use crate::email::brevo_api::send_create_contact_request;
use crate::storage::mongo::{build_customer_filter, find_customer, get_customers_collection, update_customer};
use crate::types::customer::{
    AuthProviders, Customer, Email, Preferences, PrivateSensitiveCustomer,
};
use crate::types::incoming_requests::{
    CreateCustomerRecord, CustomerUpdateName, CustomerUpdatePassword, FetchCustomerByID
};
use crate::types::state::AppState;
use crate::types::subscription::{Slug, Subscription, SubscriptionFrequencyClass};
use crate::utilities::helpers::{
    bad_request, internal_server_error, ok, parse_class, payload_analyzer, random_string, unauthorized, valid_email, valid_password
};
use crate::types::customer::GenericResponse;

use axum::extract::Query;
use axum::http::HeaderMap;
use axum::{extract::rejection::JsonRejection, http::StatusCode, Json};
use chrono::Utc;
use mongodb::bson::doc;
use serde_json::json;
use std::sync::Arc;

use bcrypt::{hash, verify, DEFAULT_COST};

use super::email::new_email_verification;
use super::identity::{get_user_session_from_req, SessionScopes};

pub async fn create_customer_record(
    payload_result: Result<Json<CreateCustomerRecord>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let payload = payload_analyzer(payload_result)?;

    if !payload.accepted_terms {
        return Err(bad_request("terms.not.accepted", None));
    }

    let auth_provider: AuthProviders;
    match payload.provider.to_lowercase().as_str() {
        "legacy" => auth_provider = AuthProviders::LEGACY,
        "google" => auth_provider = AuthProviders::GOOGLE,
        _ => auth_provider = AuthProviders::LEGACY,
    }

    if payload.name.len() < 2 || payload.name.len() > 25 {
        return Err(bad_request("invalid.name.length", None));
    }

    match valid_email(&payload.email).await {
        Ok(_) => (),
        Err(_) => return Err(bad_request("invalid.email", None)),
    };

    let mut hashed_password = "".to_string();
    if auth_provider == AuthProviders::LEGACY {
        match valid_password(&payload.password).await {
            Ok(_) => (),
            Err(_) => return Err(bad_request("invalid.password", None))
        };

        if payload.password != payload.password_confirmation {
            return Err(bad_request("invalid.password", None));
        }

        if payload.email.to_lowercase() == payload.password.to_lowercase() {
            return Err(bad_request("invalid.password", None));
        }

        hashed_password = match hash(&payload.password, DEFAULT_COST) {
            Ok(hashed_password) => hashed_password,
            Err(_) => return Err(internal_server_error("error.hashing.password", None)),
        };
    }

    let filter = build_customer_filter("", payload.email.to_lowercase().as_str()).await;
    let customer = find_customer(&state.mongo_db, filter).await;

    match customer {
        Ok(_) => return Err(bad_request("email.already.registered", None)),
        Err(_) => (),
    }

    let emails = vec![Email {
        address: payload.email.to_lowercase(),
        verified: false,
        main: true,
    }];

    let class = match parse_class(&payload.class).await {
        Ok(class) => class,
        Err(_) => return Err(bad_request("invalid.class", None)),
    };

    let current_datetime = Utc::now();
    let iso8601_string = current_datetime.to_rfc3339();
    let subscription_id = random_string(10).await;
    let subscription = Subscription {
        id: subscription_id,
        product_id: 0,
        variant_id: 0,
        slug: Slug::FREE.to_string(),
        frequency: SubscriptionFrequencyClass::UNDEFINED,
        created_at: iso8601_string.clone(),
        updated_at: iso8601_string.clone(),
        starts_at: "".to_string(),
        ends_at: "".to_string(),
        renews_at: "".to_string(),
        status: "".to_string(),
        history_logs: vec![],
    };

    let id = random_string(32).await;
    let customer = Customer {
        id,
        name: payload.name.clone(),
        class,
        emails,
        auth_provider,

        password: hashed_password,
        backup_security_codes: vec![],

        preferences: Preferences {
            dark_mode: false,
            language: String::from("en"),
            notifications: true,
        },
        subscription,

        created_at: iso8601_string.clone(),
        updated_at: iso8601_string.clone(),
        deleted: false,
        related_orgs: vec![],
    };

    let collection = get_customers_collection(&state.mongo_db).await;
    match collection.insert_one(customer.clone(), None).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }

    let created_customer_list = std::env::var("BREVO_CUSTOMERS_LIST_ID");
    let api_key = std::env::var("BREVO_CUSTOMERS_WEBFLOW_API_KEY");

    if created_customer_list.is_ok() && api_key.is_ok() {
        let created_customer_list = match created_customer_list.unwrap().parse::<u32>() {
            Ok(list_id) => list_id,
            Err(_) => 1,
        };

        let api_key = api_key.unwrap();
        match send_create_contact_request(
            &api_key,
            vec![created_customer_list],
            &customer.id,
            &customer.emails[0].address,
        )
        .await
        {
            Ok(_) => (),
            Err(_) => return Err(internal_server_error("error.sending.email", None)),
        };

        if state.enabled_email_integration {
            match new_email_verification(
                &state,
                api_key,
                customer.emails[0].address.clone(),
                customer.name.clone(),
            ).await {
                Ok(_) => (),
                Err(_) => return Err(internal_server_error("error.sending.email", None)),
            }
        }
    }

    Ok(ok("customer.created", None))
}

pub async fn fetch_customer_record_by_id(
    headers: HeaderMap,
    Query(params): Query<FetchCustomerByID>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = match get_user_session_from_req(&headers, &state.redis_connection).await {
        Ok(customer_id) => customer_id,
        Err(_) => return Err(unauthorized("invalid.token", None)),
    };

    let customer_id = match params.id {
        Some(id) => id,
        None => return Err(bad_request("invalid.id", None)),
    };

    let filter = build_customer_filter(customer_id.as_str(), "").await;
    let customer = find_customer(&state.mongo_db, filter).await?;

    let mut shared_customer_data = PrivateSensitiveCustomer {
        id: Some(customer_id),
        name: Some(customer.name),
        class: Some(customer.class),
        emails: Some(customer.emails),
        auth_provider: Some(customer.auth_provider),
        preferences: Some(customer.preferences),
        subscription: Some(customer.subscription),
        created_at: Some(customer.created_at),
        updated_at: Some(customer.updated_at),
        deleted: Some(customer.deleted),
    };

    if session_data.scopes.contains(&SessionScopes::TotalAccess) {
        return Ok(ok("customer.found", Some(json!(shared_customer_data))));
    }

    if !session_data.scopes.contains(&SessionScopes::ViewPublicID) {
        shared_customer_data.id = None;
    }

    if !session_data
        .scopes
        .contains(&SessionScopes::ViewEmailAddresses)
    {
        shared_customer_data.emails = None;
    }

    if !session_data
        .scopes
        .contains(&SessionScopes::ViewSubscription)
    {
        shared_customer_data.subscription = None;
    }

    if !session_data
        .scopes
        .contains(&SessionScopes::ViewPublicProfile)
    {
        shared_customer_data.name = None;
        shared_customer_data.class = None;
        shared_customer_data.preferences = None;
        shared_customer_data.created_at = None;
        shared_customer_data.updated_at = None;
        shared_customer_data.deleted = None;
    }

    Ok(ok("customer.found", Some(json!(shared_customer_data))))
}

pub async fn update_name(
    headers: HeaderMap,
    payload_result: Result<Json<CustomerUpdateName>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = match get_user_session_from_req(&headers, &state.redis_connection).await {
        Ok(customer_id) => customer_id,
        Err(_) => return Err(unauthorized("invalid.token", None)),
    };

    if !(session_data.scopes.contains(&SessionScopes::TotalAccess) && session_data.scopes.contains(&SessionScopes::UpdateName))
    {
        return Err(unauthorized("not.enough.scope", None));
    }

    let payload = payload_analyzer(payload_result)?;

    if payload.name.len() < 2 || payload.name.len() > 25 {
        return Err(bad_request("invalid.name.length", None));
    }

    let current_datetime = Utc::now();
    let iso8601_string = current_datetime.to_rfc3339();

    let filter = build_customer_filter(session_data.customer_id.as_str(), "").await;
    let update = doc! {"$set": {
            "name": &payload.name,
            "updated_at": iso8601_string,
        }
    };

    match update_customer(&state.mongo_db, filter, update).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }

    Ok(ok("customer.name.updated", None))
}

pub async fn update_password(
    headers: HeaderMap,
    payload_result: Result<Json<CustomerUpdatePassword>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = match get_user_session_from_req(&headers, &state.redis_connection).await {
        Ok(customer_id) => customer_id,
        Err(_) => return Err(unauthorized("invalid.token", None)),
    };

    if !session_data.scopes.contains(&SessionScopes::TotalAccess) {
        return Err(unauthorized("not.enough.scope", None));
    }

    let filter = build_customer_filter(session_data.customer_id.as_str(), "").await;
    let customer = find_customer(&state.mongo_db, filter).await?;

    let payload = payload_analyzer(payload_result)?;

    if payload.old_password.len() < 8 || payload.old_password.len() > 100 {
        return Err(bad_request("invalid.old.password.length", None));
    }

    if payload.new_password.len() < 8 || payload.new_password.len() > 100 {
        return Err(bad_request("invalid.new.password.length", None));
    }

    match valid_password(&payload.new_password).await {
        Ok(_) => (),
        Err(_) => return Err(bad_request("invalid.new.password", None)),
    };

    if payload.new_password == payload.old_password {
        return Err(bad_request("new.password.must.differ", None));
    }

    if payload.new_password != payload.new_password_confirmation {
        return Err(bad_request("password.confirmation.must.match", None));
    }

    let hashed_new_password = match hash(&payload.new_password, DEFAULT_COST) {
        Ok(hashed_password) => hashed_password,
        Err(_) => return Err(internal_server_error("error.hashing.password", None)),
    };

    match verify(&payload.old_password, &customer.password) {
        Ok(is_valid) => {
            if !is_valid {
                return Err(bad_request("invalid.old.password", None));
            }
        }
        Err(_) => return Err(internal_server_error("error.verifying.password", None)),
    };

    let current_datetime = Utc::now();
    let iso8601_string = current_datetime.to_rfc3339();

    let filter = build_customer_filter(session_data.customer_id.as_str(), "").await;
    let update = doc! {"$set": {
            "password": hashed_new_password,
            "updated_at": iso8601_string,
        }
    };

    match update_customer(&state.mongo_db, filter, update).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }

    Ok(ok("customer.password.updated", None))
}