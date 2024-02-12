use std::sync::{Arc, MutexGuard};

use axum::{extract::rejection::JsonRejection, http::StatusCode, Json};
use rust_bert::pipelines::sequence_classification::SequenceClassificationModel;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{storage::mongo::{build_organizations_filter, find_organization}, types::{customer::GenericResponse, incoming_requests::ProcessPrompt, organization::ModelObject, state::AppState}, utilities::{api_messages::{APIMessages, LLMRouterMessages}, helpers::{detect_similar_sentences, payload_analyzer}}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProccesedPrompt {
    pub single_model_used: bool,
    pub single_model_output: Option<ModelObject>,

    pub prompt_calification_model_used: bool,
    pub prompt_calification_model_output: Option<ModelObject>,
    pub prompt_calification_model_output_precision: f64,

    pub sentence_match_used: bool,
    pub exact_sentence_match_used: bool,
    pub cosine_similarity_sentence_match_used: bool,
    pub cosine_similarity_score: f32,
    pub cosine_similarity_temperature: f32,
    pub sentence_match_output: Option<ModelObject>,

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
                    message: "required organization_id".to_string(),
                    data: json!({}),
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
                    message: "required router_id".to_string(),
                    data: json!({}),
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
                    message: APIMessages::LLMRouter(LLMRouterMessages::RequiredPromptField).to_string(),
                    data: json!({}),
                    exit_code: 1,
                }),
            );
        }
    };

    if org_id.is_empty() || org_id.len() != 32 {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: "invalid org id".to_string(),
                data: json!({}),
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
                message: "organization not found".to_string(),
                data: json!({}),
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
                    message: "router not found".to_string(),
                    data: json!({}),
                    exit_code: 1,
                }),
            );
        }
    };

    if router.active == false || router.deleted == true {
        return (
            StatusCode::FORBIDDEN,
            Json(GenericResponse {
                message: "unavailable router".to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        );
    }

    if router.use_single_model {
        return (
            StatusCode::OK,
            Json(GenericResponse {
                message: "proccesed".to_string(),
                data: json!({
                    "proccesed_prompt_result": ProccesedPrompt {
                        single_model_used: true,
                        single_model_output: Some(router.model),
                        prompt_calification_model_used: false,
                        prompt_calification_model_output: None,
                        prompt_calification_model_output_precision: 0.0,
                        sentence_match_used: false,
                        exact_sentence_match_used: false,
                        cosine_similarity_sentence_match_used: false,
                        cosine_similarity_score: 0.0,
                        cosine_similarity_temperature: 0.0,
                        sentence_match_output: None,
                        prompt: prompt.to_string(),
                        prompt_size: prompt.len().try_into().unwrap(),
                    },
                }),
                exit_code: 0,
            }),
        );
    }

    if prompt.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: APIMessages::LLMRouter(LLMRouterMessages::RequiredPromptField).to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        );
    }

    if prompt.len() > router.max_prompt_length.try_into().unwrap() || prompt.len() < 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(GenericResponse {
                message: format!(
                    "prompt length must be between 1 and {}",
                    router.max_prompt_length
                ),
                data: json!({}),
                exit_code: 1,
            }),
        );
    }

    if router.use_prompt_calification_model {
        let model: MutexGuard<'_, SequenceClassificationModel> = state.llm_resources.prompt_classification_model.model.lock().unwrap();
        let input = [payload.prompt.as_deref().unwrap_or_default()];
        let output = model.predict(&input);
        drop(model);

        for category in router.prompt_calification_model_categories {
            if category.label == output[0].text {
                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "proccesed".to_string(),
                        data: json!({
                            "proccesed_prompt_result": ProccesedPrompt {
                                single_model_used: false,
                                single_model_output: None,
                                prompt_calification_model_used: true,
                                prompt_calification_model_output: Some(category.model),
                                prompt_calification_model_output_precision: output[0].score,
                                sentence_match_used: false,
                                exact_sentence_match_used: false,
                                cosine_similarity_sentence_match_used: false,
                                cosine_similarity_score: 0.0,
                                cosine_similarity_temperature: 0.0,
                                sentence_match_output: None,
                                prompt: prompt.to_string(),
                                prompt_size: prompt.len().try_into().unwrap(),
                            },
                        }),
                        exit_code: 0,
                    }),
                );
            }


            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: "model output a unknown value".to_string(),
                    data: json!({}),
                    exit_code: 1,
                }),
            );
        }
    }

    if router.use_sentence_matching {
        for sentence in router.sentences {
            if sentence.exact && sentence.text.to_lowercase() == prompt.to_lowercase() {
                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "proccesed".to_string(),
                        data: json!({
                            "proccesed_prompt_result": ProccesedPrompt {
                                single_model_used: false,
                                single_model_output: None,
                                prompt_calification_model_used: false,
                                prompt_calification_model_output: None,
                                prompt_calification_model_output_precision: 0.0,
                                sentence_match_used: true,
                                exact_sentence_match_used: true,
                                cosine_similarity_sentence_match_used: false,
                                cosine_similarity_score: 0.0,
                                cosine_similarity_temperature: 0.0,
                                sentence_match_output: Some(sentence.model),
                                prompt: prompt.to_string(),
                                prompt_size: prompt.len().try_into().unwrap(),
                            },
                        }),
                        exit_code: 0,
                    }),
                );
            } else if sentence.use_cosine_similarity {
                // embedding model
                let (similar, score) = match detect_similar_sentences(&state.llm_resources.embedding_model, sentence.text, prompt.to_string(), sentence.cosine_similarity_temperature).await {
                    Ok((similar, score)) => (similar, score),
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(GenericResponse {
                                message: "error detecting similar sentences".to_string(),
                                data: json!({}),
                                exit_code: 1,
                            }),
                        );
                    }
                };

                if !similar {
                    continue;
                }

                return (
                    StatusCode::OK,
                    Json(GenericResponse {
                        message: "proccesed".to_string(),
                        data: json!({
                            "proccesed_prompt_result": ProccesedPrompt {
                                single_model_used: false,
                                single_model_output: None,
                                prompt_calification_model_used: false,
                                prompt_calification_model_output: None,
                                prompt_calification_model_output_precision: 0.0,
                                sentence_match_used: true,
                                exact_sentence_match_used: false,
                                cosine_similarity_sentence_match_used: true,
                                cosine_similarity_score: score,
                                cosine_similarity_temperature: sentence.cosine_similarity_temperature,
                                sentence_match_output: Some(sentence.model),
                                prompt: prompt.to_string(),
                                prompt_size: prompt.len().try_into().unwrap(),
                            },
                        }),
                        exit_code: 0,
                    }),
                );
                
            }
        }

        return (
            StatusCode::OK,
            Json(GenericResponse {
                message: "prompt doesn't match any sentence".to_string(),
                data: json!({}),
                exit_code: 1,
            }),
        );
    }

    return (
        StatusCode::OK,
        Json(GenericResponse {
            message: "not proccesed".to_string(),
            data: json!({
                "reason": "not_method_to_proccesed".to_string(),
            }),
            exit_code: 0,
        }),
    );
}