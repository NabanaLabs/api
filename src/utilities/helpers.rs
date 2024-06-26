use std::sync::{Arc, Mutex};

use crate::types::{customer::{GenericResponse, CustomerType}, subscription::SubscriptionHistoryLog};
use axum::{
    extract::rejection::JsonRejection,
    http::{StatusCode, Uri},
    Json,
};
use mongodb::bson::{to_document, Document};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use rust_bert::{pipelines::sentence_embeddings::SentenceEmbeddingsModel, RustBertError};
use serde_json::{json, Value};

use super::api_messages::{APIMessages, CustomerMessages, EmailMessages, InputMessages};

pub fn payload_analyzer<T>(
    payload_result: Result<Json<T>, JsonRejection>,
) -> Result<Json<T>, (StatusCode, Json<GenericResponse>)> {
    let payload = match payload_result {
        Ok(payload) => payload,
        Err(_) => return Err(bad_request("invalid.payload", None))
    };

    Ok(payload)
}

pub async fn fallback(uri: Uri) -> (StatusCode, Json<GenericResponse>) {
    let message = format!("invalid.endpoint.{}", uri.path());
    (
        StatusCode::NOT_FOUND,
        Json(GenericResponse {
            message,
            data: json!({}),
            exit_code: 1,
        }),
    )
}

pub async fn random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub async fn valid_email(email: &String) -> Result<bool, (StatusCode, Json<GenericResponse>)> {
    if  email.len() < 5 || email.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::Email(EmailMessages::Invalid).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        ));
    }

    let re = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !re.is_match(email.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::Email(EmailMessages::Invalid).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        ));
    };
    
    Ok(true)
}

pub async fn valid_password(password: &String) -> Result<bool, (StatusCode, Json<GenericResponse>)> {
    if password.len() < 8 || password.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::Input(InputMessages::InvalidNewPasswordLength).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        ));
    }

    let re = Regex::new(r"^[a-zA-Z0-9_]{8,20}$").unwrap();
    if !re.is_match(password.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::Input(InputMessages::PasswordMustHaveAtLeastOneLetterAndOneNumber).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        ));
    };

    Ok(true)
}

pub async fn parse_class(raw_class: &String) -> Result<CustomerType, (StatusCode, Json<GenericResponse>)> {
    let class: CustomerType;
    if raw_class.to_lowercase() == "personal" {
        class = CustomerType::PERSONAL;
    } else if raw_class.to_lowercase() == "business" {
        class = CustomerType::BUSINESS;
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::Customer(CustomerMessages::InvalidType).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        ));
    }

    return Ok(class)
}

pub async fn add_subscription_history_log_and_to_bson(mut history_logs: Vec<SubscriptionHistoryLog>, log: SubscriptionHistoryLog) -> Vec<Document> {
    history_logs.push(log);
    let bson_history_logs: Vec<Document> = history_logs.iter()
    .map(|log| {
        match to_document(log) {
            Ok(document) => document,
            Err(_) => {
                return Document::new();
            }
        }
    })
    .collect();

    return bson_history_logs;
}

// LLM

pub fn calculate_cosine_similarity(embedding1: Vec<f32>, embedding2: Vec<f32>) -> f32 {
    let dot_product = embedding1.iter().zip(embedding2.iter()).map(|(a, b)| a * b).sum::<f32>();
    let norm1 = embedding1.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let norm2 = embedding2.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();

    dot_product / (norm1 * norm2)
}

pub async fn detect_similar_sentences(model: &Arc<Box<Mutex<SentenceEmbeddingsModel>>>, sentence_one: String, sentence_two: String, temperature: f32) -> Result<(bool, f32), RustBertError> {
    let model = model.lock().unwrap();
    let sentences = [sentence_one, sentence_two];
    let embeddings = model.encode(&sentences)?;
    drop(model);
    // Calculate cosine similarity
    let similarity = calculate_cosine_similarity(embeddings[0].clone(), embeddings[1].clone());
    if similarity.is_nan() {
        return Ok((false, similarity));
    }

    if similarity.is_infinite() {
        return Ok((false, similarity));
    }

    if similarity < temperature {
        return Ok((false, similarity));
    }

    return Ok((true, similarity));
}

// response helpers

pub fn ok(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "ok",
        _ => message,
    };

    (
        StatusCode::OK,
        Json(GenericResponse {
            message: message.to_string(),
            data,
            exit_code: 0,
        }),
    )
}

pub fn internal_server_error(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "internal.server.error",
        _ => message,
    };

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(GenericResponse {
            message: message.to_string(),
            data,
            exit_code: 1,
        }),
    )
}

pub fn bad_request(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "bad.request",
        _ => message,
    };

    (
        StatusCode::BAD_REQUEST,
        Json(GenericResponse {
            message: message.to_string(),
            data,
            exit_code: 1,
        }),
    )
}

pub fn not_found(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "not.found",
        _ => message,
    };
    
    (
        StatusCode::NOT_FOUND,
        Json(GenericResponse {
            message: message.to_string(),
            data: data,
            exit_code: 1,
        }),
    )
}

pub fn unauthorized(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "unauthorized",
        _ => message,
    };
    
    (
        StatusCode::UNAUTHORIZED,
        Json(GenericResponse {
            message: message.to_string(),
            data,
            exit_code: 1,
        }),
    )
}

#[allow(dead_code)]
pub fn forbidden(message: &str, data: Option<Value>) -> (StatusCode, Json<GenericResponse>) {
    let data = match data {
        Some(data) => data,
        None => json!({}),
    };

    let message = match message {
        "" => "forbidden",
        _ => message,
    };
    
    (
        StatusCode::FORBIDDEN,
        Json(GenericResponse {
            message: message.to_string(),
            data,
            exit_code: 1,
        }),
    )
}
