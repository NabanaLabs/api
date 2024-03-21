use axum::http::HeaderMap;
use axum::{http::StatusCode, Json};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use redis::Commands;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::controllers::identity::{get_all_scopes, SessionScopes};
use crate::types::customer::GenericResponse;

use super::api_messages::{APIMessages, TokenMessages};
use super::helpers::{bad_request, internal_server_error, unauthorized};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: usize,
}

pub fn scopes_to_string(scopes: Vec<SessionScopes>) -> String {
    let sanitized_scopes = scopes
        .iter()
        .map(|scope| scope.to_string())
        .collect::<Vec<String>>();

    sanitized_scopes.join(",")
}

pub fn string_to_scopes(scopes: String) -> Vec<SessionScopes> {
    let sanitized_scopes = scopes
        .split(",")
        .map(|scope| scope.parse::<SessionScopes>().unwrap())
        .collect::<Vec<SessionScopes>>();

    sanitized_scopes
}

pub fn create_token(id: &String, scopes: Vec<SessionScopes>) -> Result<std::string::String, String> {
    let api_url = env::var("API_URL").unwrap_or(String::from("http://localhost:3000"));
    let expiration_time = env::var("API_TOKENS_EXPIRATION_TIME").unwrap_or(String::from("86400"));
    let header = Header::new(Algorithm::HS512);

    let sanitized_scopes = scopes_to_string(scopes);

    let claims = Claims {
        iss: api_url,
        sub: id.to_string(),
        aud: sanitized_scopes,
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + expiration_time.parse::<usize>().unwrap(),
    };

    let signing_key = match env::var("API_TOKENS_SIGNING_KEY") {
        Ok(key) => key,
        Err(_) => return Err(APIMessages::Token(TokenMessages::NotSigningKeyFound).to_string()),
    };

    match encode(
        &header,
        &claims,
        &EncodingKey::from_secret(signing_key.as_ref()),
    ) {
        Ok(t) => Ok(t),
        Err(_) => Err(APIMessages::Token(TokenMessages::ErrorCreating).to_string()),
    }
}

pub fn get_token_payload(token: &str) -> Result<TokenData<Claims>, (StatusCode, Json<GenericResponse>)> {
    let scopes = get_all_scopes();
    let mut audience: Vec<&str> = Vec::new();
    for scope in &scopes {
        audience.push(scope);
    }
    
    let mut validation = Validation::new(Algorithm::HS512);
    validation.set_audience(&audience);

    let signing_key = match env::var("API_TOKENS_SIGNING_KEY") {
        Ok(key) => key,
        Err(_) => return Err(bad_request("token.signing_key", None)),
    };

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(signing_key.as_ref()),
        &validation,
    ) {
        Ok(t) => t,
        Err(_) => return Err(unauthorized("token.invalid", None)),
    };

    Ok(token_data)
}

pub fn validate_token(token: &str) -> Result<TokenData<Claims>, (StatusCode, Json<GenericResponse>)> {
    let token_data = get_token_payload(token)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    if now.as_secs() > token_data.claims.exp as u64 {
        return Err(unauthorized("token.expired", None));
    }

    Ok(token_data)
}

pub async fn get_session_from_redis(
    redis_connection: &Client,
    token_string: &str,
) -> Result<String, (StatusCode, Json<GenericResponse>)> {
    let result = redis_connection.clone().get::<String, String>(token_string.to_string());

    match result {
        Ok(id) => Ok(id),
        Err(_) => return Err(internal_server_error("invalid.token", None)),
    }
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Result<&str, (StatusCode, Json<GenericResponse>)> {
    match headers.get("Authorization") {
        Some(token) => match token.to_str() {
            Ok(token) => Ok(token),
            Err(_) => Err(internal_server_error("", None)),
        },
        None => Err(unauthorized("", None)),
    }
}
