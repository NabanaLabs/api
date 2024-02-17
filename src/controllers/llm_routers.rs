use std::sync::{Arc, MutexGuard};

use axum::{extract::rejection::JsonRejection, http::StatusCode, Json};
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    storage::mongo::{build_organizations_filter, find_organization},
    types::{
        customer::GenericResponse, incoming_requests::ProcessPrompt, llm_router::Category,
        organization::ModelObject, state::AppState,
    },
    utilities::{
        api_messages::{APIMessages, LLMRouterMessages},
        helpers::{detect_similar_sentences, payload_analyzer},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptClassification {
    pub used: bool,
    pub label: Option<String>,
    pub precision: Option<f64>,
    pub model: Option<ModelObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleModel {
    pub used: bool,
    pub model: Option<ModelObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceMatching {
    pub used: bool,
    pub exact: bool,
    pub cosine_similarity: bool,
    pub similarity_level: Option<f32>,
    pub temperature: Option<f32>,
    pub appropiate_match: bool,
    pub model: Option<ModelObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProccesedPrompt {
    pub single_model: Option<SingleModel>,
    pub prompt_calification: Option<PromptClassification>,
    pub sentence_matching: Option<SentenceMatching>,

    pub prompt: String,
    pub prompt_size: i32,
}

pub async fn process_prompt(
    payload_result: Result<Json<ProcessPrompt>, JsonRejection>,
    state: Arc<AppState>,
) -> (StatusCode, Json<GenericResponse>) {
    let payload = match payload_analyzer(payload_result) {
        Ok(payload) => payload,
        Err((status_code, json)) => return (status_code, json),
    };

    let org_id = match payload.organization_id.as_deref() {
        Some(org_id) => org_id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(GenericResponse {
                    message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                    data: json!({
                        "reason": "required.org.id".to_string(),
                    }),
                    exit_code: 1,
                }),
            );
        }
    };

    let router_id = match payload.router_id.as_deref() {
        Some(router_id) => router_id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(GenericResponse {
                    message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                    data: json!({
                        "reason": "required.router.id".to_string(),
                    }),
                    exit_code: 1,
                }),
            );
        }
    };

    let prompt = match payload.prompt.as_deref() {
        Some(prompt) => prompt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(GenericResponse {
                    message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                    data: json!({
                        "reason": APIMessages::LLMRouter(LLMRouterMessages::RequiredPromptField).to_string(),
                    }),
                    exit_code: 1,
                }),
            );
        }
    };

    if org_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                data: json!({
                    "reason": "invalid.org.id".to_string(),
                }),
                exit_code: 1,
            }),
        );
    }

    let filter = build_organizations_filter(org_id).await;
    let (found, org) = match find_organization(&state.mongo_db, filter).await {
        Ok((found, customer)) => (found, customer),
        Err((status_code, json)) => return (status_code, json),
    };

    if !found {
        return (
            StatusCode::NOT_FOUND,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                data: json!({
                    "reason": "org.not.found".to_string(),
                }),
                exit_code: 1,
            }),
        );
    }

    let org = org.unwrap();

    let mut router = None;
    for r in org.routers {
        if r.id == router_id {
            router = Some(r);
            break;
        }
    }

    let router = match router {
        Some(router) => router,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(GenericResponse {
                    message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                    data: json!({
                        "reason": "router.not.found".to_string(),
                    }),
                    exit_code: 1,
                }),
            );
        }
    };

    if router.active == false || router.deleted == true {
        return (
            StatusCode::FORBIDDEN,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                data: json!({
                    "reason": "unavailable.router".to_string(),
                }),
                exit_code: 1,
            }),
        );
    }

    if router.use_single_model {
        let selected_model_object =
            match org.models.iter().find(|model| model.id == router.model_id) {
                Some(model) => model,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(GenericResponse {
                            message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed)
                                .to_string(),
                            data: json!({
                                "reason": "model not found".to_string(),
                            }),
                            exit_code: 1,
                        }),
                    );
                }
            };

        return (
            StatusCode::OK,
            Json(GenericResponse {
                message: "ok".to_string(),
                data: json!(ProccesedPrompt {
                    single_model: Some(SingleModel {
                        used: true,
                        model: Some(selected_model_object.clone()),
                    }),
                    prompt_calification: None,
                    sentence_matching: None,
                    prompt: prompt.to_string(),
                    prompt_size: prompt.len().try_into().unwrap(),
                }),
                exit_code: 0,
            }),
        );
    }

    if prompt.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                data: json!({
                    "reason": "required prompt field".to_string(),
                }),
                exit_code: 1,
            }),
        );
    }

    if prompt.len() > router.max_prompt_length.try_into().unwrap() || prompt.len() < 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                data: json!({
                    "reason": format!(
                        "prompt length must be between 1 and {}",
                        router.max_prompt_length
                    ),
                }),
                exit_code: 1,
            }),
        );
    }

    if router.use_prompt_calification_model {
        let model: MutexGuard<'_, ZeroShotClassificationModel> = state
            .llm_resources
            .prompt_classification_model
            .model
            .lock()
            .unwrap();
        let input = [payload.prompt.as_deref().unwrap_or_default()];

        let router_categories: &[Category] = &router.prompt_calification_model_categories;
        let candidate_labels: Vec<&str> = router_categories
            .iter()
            .map(|category| category.label.as_str())
            .collect();

        let output = match model.predict_multilabel(
            input,
            candidate_labels,
            Some(Box::new(|label: &str| format!("{label}"))),
            128,
        ) {
            Ok(output) => output,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(GenericResponse {
                        message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed)
                            .to_string(),
                        data: json!({
                            "reason": "model couldn't predict".to_string(),
                        }),
                        exit_code: 1,
                    }),
                );
            }
        };

        let prompt_output = output[0].clone();
        drop(model);

        let (label_text, score) = prompt_output.iter().fold(("", 0.0), |acc, label| {
            if label.score > acc.1 {
                (label.text.as_str(), label.score)
            } else {
                acc
            }
        });

        for category in router.prompt_calification_model_categories {
            if category.label == label_text {
                let selected_model_object = match org
                    .models
                    .iter()
                    .find(|model| model.id == category.model_id)
                {
                    Some(model) => model,
                    None => {
                        return (
                            StatusCode::NOT_FOUND,
                            Json(GenericResponse {
                                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed)
                                    .to_string(),
                                data: json!({
                                    "reason": "category specified model not found in mode list".to_string(),
                                }),
                                exit_code: 1,
                            }),
                        );
                    }
                };

                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "ok".to_string(),
                        data: json!(ProccesedPrompt {
                            single_model: None,
                            prompt_calification: Some(PromptClassification {
                                used: true,
                                label: Some(label_text.to_string()),
                                precision: Some(score),
                                model: Some(selected_model_object.clone()),
                            }),
                            sentence_matching: None,
                            prompt: prompt.to_string(),
                            prompt_size: prompt.len().try_into().unwrap(),
                        }),
                        exit_code: 0,
                    }),
                );
            }

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                    data: json!({
                        "reason": "unknown label category".to_string(),
                    }),
                    exit_code: 1,
                }),
            );
        }
    }

    if router.use_sentence_matching {
        for (index, sentence) in router.sentences.iter().enumerate() {
            let selected_model_object = match org
                .models
                .iter()
                .find(|model| model.id == sentence.model_id)
            {
                Some(model) => model,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(GenericResponse {
                            message: "model not found".to_string(),
                            data: json!({}),
                            exit_code: 1,
                        }),
                    );
                }
            };

            if sentence.exact && sentence.text.to_lowercase() == prompt.to_lowercase() {
                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "ok".to_string(),
                        data: json!(ProccesedPrompt {
                            single_model: None,
                            prompt_calification: None,
                            sentence_matching: Some(SentenceMatching {
                                used: true,
                                exact: true,
                                cosine_similarity: false,
                                similarity_level: None,
                                temperature: None,
                                appropiate_match: true,
                                model: Some(selected_model_object.clone()),
                            }),
                            prompt: prompt.to_string(),
                            prompt_size: prompt.len().try_into().unwrap(),
                        }),
                        exit_code: 0,
                    }),
                );
            } else if sentence.use_cosine_similarity {
                // embedding model
                let (similar, score) = match detect_similar_sentences(
                    &state.llm_resources.embedding_model,
                    sentence.text.clone(),
                    prompt.to_string(),
                    sentence.cosine_similarity_temperature,
                )
                .await
                {
                    Ok((similar, score)) => (similar, score),
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(GenericResponse {
                                message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
                                data: json!({
                                    "reason": "error detecting similar sentences".to_string(),
                                }),
                                exit_code: 1,
                            }),
                        );
                    }
                };

                if index == router.sentences.len() - 1 && !similar {
                    return (
                        StatusCode::OK,
                        Json(GenericResponse {
                            message: "not matched any sentence propertly".to_string(),
                            data: json!(ProccesedPrompt {
                                single_model: None,
                                prompt_calification: None,
                                sentence_matching: Some(SentenceMatching {
                                    used: true,
                                    exact: false,
                                    cosine_similarity: true,
                                    similarity_level: Some(score),
                                    temperature: Some(sentence.cosine_similarity_temperature),
                                    appropiate_match: similar,
                                    model: Some(selected_model_object.clone()),
                                }),
                                prompt: prompt.to_string(),
                                prompt_size: prompt.len().try_into().unwrap(),
                            }),
                            exit_code: 1,
                        }),
                    );
                }

                if !similar {
                    continue;
                }

                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "ok".to_string(),
                        data: json!(ProccesedPrompt {
                            single_model: None,
                            prompt_calification: None,
                            sentence_matching: Some(SentenceMatching {
                                used: true,
                                exact: false,
                                cosine_similarity: true,
                                similarity_level: Some(score),
                                temperature: Some(sentence.cosine_similarity_temperature),
                                appropiate_match: similar,
                                model: Some(selected_model_object.clone()),
                            }),
                            prompt: prompt.to_string(),
                            prompt_size: prompt.len().try_into().unwrap(),
                        }),
                        exit_code: 0,
                    }),
                );
            }
        }
    }

    return (
        StatusCode::OK,
        Json(GenericResponse {
            message: APIMessages::LLMRouter(LLMRouterMessages::NotProccesed).to_string(),
            data: json!({
                "reason": "not method select to proccess prompt".to_string(),
            }),
            exit_code: 0,
        }),
    );
}