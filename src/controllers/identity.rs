use crate::oauth::google::{get_google_user, request_token};
use crate::types::state::AppState;
use crate::utilities::helpers::{bad_request, internal_server_error, not_found, ok, payload_analyzer, unauthorized};
use crate::storage::mongo::{build_customer_filter, find_customer};
use crate::utilities::token::{create_token, extract_token_from_headers, get_session_from_redis, get_token_payload, string_to_scopes, validate_token};
use crate::types::customer::{AuthProviders, GenericResponse};
use crate::types::incoming_requests::SignIn;

use axum::extract::Query;
use axum::http::HeaderMap;
use axum::{
    extract::rejection::JsonRejection, 
    http::StatusCode, Json
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

use bcrypt::verify;
use redis::{Client, Commands, RedisError};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionScopes {
    ViewPublicID,
    ViewEmailAddresses,
    ViewPublicProfile,
    ViewPrivateSensitiveProfile,
    ViewSubscription,
    
    UpdateName,
    UpdateEmailAddresses,
    UpdatePreferences,

    ManageOrganizations,

    TotalAccess, // never use this for 3rd party apps
}

impl ToString for SessionScopes {
    fn to_string(&self) -> String {
        match self {
            SessionScopes::ViewPublicID => String::from("view_public_id"),
            SessionScopes::ViewEmailAddresses => String::from("view_email_addresses"),
            SessionScopes::ViewPublicProfile => String::from("view_public_profile"),
            SessionScopes::ViewPrivateSensitiveProfile => String::from("view_private_sensitive_profile"),
            SessionScopes::ViewSubscription => String::from("view_subscription"),
            
            SessionScopes::UpdateName => String::from("update_name"),
            SessionScopes::UpdateEmailAddresses => String::from("update_email_addresses"),
            SessionScopes::UpdatePreferences => String::from("update_preferences"),

            SessionScopes::ManageOrganizations => String::from("manage_organizations"),

            SessionScopes::TotalAccess => String::from("total_access"),
        }
    }
}

impl FromStr for SessionScopes {
    type Err = ();

    fn from_str(input: &str) -> Result<SessionScopes, Self::Err> {
        match input {
            "view_public_id" => Ok(SessionScopes::ViewPublicID),
            "view_email_addresses" => Ok(SessionScopes::ViewEmailAddresses),
            "view_public_profile" => Ok(SessionScopes::ViewPublicProfile),
            "view_private_sensitive_profile" => Ok(SessionScopes::ViewPrivateSensitiveProfile),
            "view_subscription" => Ok(SessionScopes::ViewSubscription),
            
            "update_name" => Ok(SessionScopes::UpdateName),
            "update_email_addresses" => Ok(SessionScopes::UpdateEmailAddresses),
            "update_preferences" => Ok(SessionScopes::UpdatePreferences),

            "manage_organizations" => Ok(SessionScopes::ManageOrganizations),

            "total_access" => Ok(SessionScopes::TotalAccess),
            _ => Err(()),
        }
    }
}

pub fn get_all_scopes() -> Vec<String> {
    vec![
        SessionScopes::ViewPublicID.to_string(),
        SessionScopes::ViewEmailAddresses.to_string(),
        SessionScopes::ViewPublicProfile.to_string(),
        SessionScopes::ViewPrivateSensitiveProfile.to_string(),
        SessionScopes::ViewSubscription.to_string(),
        SessionScopes::UpdateName.to_string(),
        SessionScopes::UpdateEmailAddresses.to_string(),
        SessionScopes::UpdatePreferences.to_string(),
        SessionScopes::ManageOrganizations.to_string(),
        SessionScopes::TotalAccess.to_string(),
    ]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub customer_id: String,
    pub scopes: Vec<SessionScopes>,
}

pub async fn get_user_session_from_req(
    headers: &HeaderMap,
    redis_connection: &Client,
) -> Result<SessionData, (StatusCode, Json<GenericResponse>)> {
    let token_string = extract_token_from_headers(&headers)?;
    validate_token(token_string)?;

    let customer_id = get_session_from_redis(redis_connection, &token_string).await?;
    let token_data = get_token_payload(&token_string)?;

    if customer_id != token_data.claims.sub {
        return Err(bad_request("invalid.token", None));
    }

    let raw_scopes = token_data.claims.aud;
    let scopes: Vec<SessionScopes> = string_to_scopes(raw_scopes);
    
    let session_data = SessionData {
        customer_id,
        scopes,
    };

    return Ok(session_data);
}

pub async fn get_session(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = match get_user_session_from_req(&headers, &state.redis_connection).await {
        Ok(id) => id,
        Err(_) => return Err(unauthorized("", None)),
    };

    if session_data.customer_id.is_empty() {
        return Err(unauthorized("", None));
    }

    return Ok(ok("", Some(json!({
        "customer_id": session_data.customer_id,
        "scopes": session_data.scopes,
    }))));
}

pub async fn renew_session(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let token_string = extract_token_from_headers(&headers)?;

    let mut redis_conn = match state.redis_connection.get_connection() {
        Ok(redis_conn) => redis_conn,
        Err(_) => return Err(unauthorized("", None)),
    };

    let customer_id: String = match redis_conn.get(token_string.to_string()) {
        Ok(customer_id) => customer_id,
        Err(_) => return Err(unauthorized("", None)),
    };

    if customer_id.is_empty() {
        return Err(unauthorized("", None));
    }

    let result: Result<bool, RedisError> =
        redis_conn
            .set_ex(token_string.to_string(), customer_id.clone(), 604800);

    match result {
        Ok(_) => (),
        Err(_) => return Err(unauthorized("", None)),
    };

    return Ok(ok("session.renewed", Some(json!({
        "customer_id": customer_id,
    }))));
}

pub async fn legacy_authentication(
    payload_result: Result<Json<SignIn>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let payload = match payload_analyzer(payload_result) {
        Ok(payload) => payload,
        Err(_) => return Err(bad_request("", None))
    };

    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    if !email_re.is_match(&payload.email) {
        return Err(bad_request("invalid.email", None));
    }

    let filter = build_customer_filter("", payload.email.as_str()).await;
    let customer = find_customer(&state.mongo_db, filter).await?;

    if customer.auth_provider != AuthProviders::LEGACY {
        return Err(unauthorized("invalid.auth.provider", None));
    }

    match verify(&payload.password, &customer.password) {
        Ok(is_valid) => {
            if !is_valid {
                return Err(unauthorized("invalid.credentials", None));
            }
        },
        Err(_) => return Err(internal_server_error("password.error", None)),
    };

    if customer.auth_provider != AuthProviders::LEGACY {
        return Err(unauthorized("invalid.auth.provider", None));
    }

    let token = match create_token(&customer.id, vec![SessionScopes::TotalAccess]) {
        Ok(token) => token,
        Err(_) => return Err(internal_server_error("token.error", None)),
    };

    let mut redis_conn = match state.redis_connection.get_connection() {
        Ok(redis_conn) => redis_conn,
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    let result: Result<bool, RedisError> =
        redis_conn
            .set_ex(token.clone(), &customer.id, 604800);

    match result {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    return Ok(ok("session.created", Some(json!({
        "token": token,
    }))));
}

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthQueryParams {
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn gooogle_authentication(
    Query(params): Query<GoogleOAuthQueryParams>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    match params.error {
        Some(_) => return Err(unauthorized("google.error", None)),
        None => (),
    };

    let authorization_code = match params.code {
        Some(token) => token,
        None => return Err(bad_request("google.error", None)),
    };

    let token_response = match request_token(&authorization_code, &state).await {
        Ok(token_response) => token_response,
        Err(_) => return Err(internal_server_error("google.error", None)),
    };
    
    let google_user = match get_google_user(&token_response.access_token, &token_response.id_token).await {
        Ok(google_user) => google_user,
        Err(_) => return Err(internal_server_error("google.error", None)),
    
    };

    let google_user_email = match google_user.email {
        Some(email) => email,
        None => return Err(bad_request("google.error", None)),
    };

    let filter = build_customer_filter("", &google_user_email).await;
    let customer = match find_customer(&state.mongo_db, filter).await {
        Ok(customer) => customer,
        Err(_) => {
            return Err(not_found("customer.not.found", Some(json!({
                "action": "create_customer_record",
                "auth_provider": AuthProviders::GOOGLE,
                "openid": google_user.id,
                "email": google_user_email,
                "verified_email": google_user.verified_email,
                "name": google_user.name,
                "given_name": google_user.given_name,
                "family_name": google_user.family_name,
                "picture": google_user.picture,
                "locale": google_user.locale,
            }))));
        }
    };

    if customer.auth_provider != AuthProviders::GOOGLE {
        return Err(unauthorized("invalid.auth.provider", None));
    }

    let token = match create_token(&customer.id, vec![SessionScopes::TotalAccess]) {
        Ok(token) => token,
        Err(_) => return Err(internal_server_error("token.error", None)),
    };

    let mut redis_conn = match state.redis_connection.get_connection() {
        Ok(redis_conn) => redis_conn,
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    let result: Result<bool, RedisError> =
        redis_conn
            .set_ex(token.clone(), &customer.id, 604800);

    match result {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("cache.error", None)),
    };

    return Ok(ok("session.created", Some(json!({
        "token": token,
    }))));
}
