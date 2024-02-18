use std::sync::{Arc, MutexGuard};

use crate::{
    storage::mongo::{build_organizations_filter, find_organization},
    types::{
        customer::GenericResponse,
        incoming_requests::ProcessPrompt,
        llm_router::Category,
        llms::LLMs,
        openai_models::OpenAIModels,
        organization::{AccessTokenScopes, ModelObject},
        state::AppState,
    },
    utilities::helpers::{
        bad_request, detect_similar_sentences, internal_server_error, ok, payload_analyzer,
        unauthorized,
    },
};
use axum::{
    extract::rejection::JsonRejection,
    http::{HeaderMap, StatusCode},
    Json,
};
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;
use serde::{Deserialize, Serialize};

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
    headers: HeaderMap,
    payload_result: Result<Json<ProcessPrompt>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let payload = payload_analyzer(payload_result)?;

    let org_id = match headers.get("OrganizationID") {
        Some(orgid) => orgid,
        None => return Err(unauthorized("organization.id", None)),
    };

    let router_id = match headers.get("RouterID") {
        Some(routerid) => routerid,
        None => return Err(unauthorized("router.id", None)),
    };

    let org_access_token = match headers.get("Authorization") {
        Some(routerid) => routerid,
        None => return Err(unauthorized("", None)),
    };

    if org_id.is_empty() || router_id.is_empty() || payload.prompt.is_none() {
        return Err(bad_request("invalid.payload.and.or.headers", None));
    }

    let router_id = router_id.to_str().unwrap();
    let org_id = org_id.to_str().unwrap();
    let org_access_token = org_access_token.to_str().unwrap();
    let prompt = payload.prompt.as_deref().unwrap();

    let filter = build_organizations_filter(org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    let mut authorized = false;
    for access_token in &org.access_tokens {
        if access_token.token == org_access_token {
            authorized = true;
            break;
        }
    }

    if !authorized {
        return Err(unauthorized("unauthorized.access.token", None));
    }

    let required_scope: Vec<String> = vec![
        AccessTokenScopes::AccessPromptModelSuggestion.to_string(),
        AccessTokenScopes::Admin.to_string(),
    ];

    if !org.access_tokens.iter().any(|access_token| {
        access_token.token == org_access_token && 
        access_token.scopes.iter().any(|scope| required_scope.contains(&scope.to_string()))
    }) {
        return Err(unauthorized("unauthorized.access.token.scopes", None));
    }

    let router = match org.routers.into_iter().find(|r| r.id == router_id) {
        Some(router) => {
            if router.active == false || router.deleted == true {
                return Err(bad_request("router.not.found", None));
            }

            router
        }
        None => {
            return Err(bad_request("router.not.found", None));
        }
    };

    if router.use_single_model {
        let selected_model_object =
            match org.models.iter().find(|model| model.id == router.model_id) {
                Some(model) => model,
                None => {
                    return Err(bad_request("model.not.found", None));
                }
            };

        let data = ProccesedPrompt {
            single_model: Some(SingleModel {
                used: true,
                model: Some(selected_model_object.clone()),
            }),
            prompt_calification: None,
            sentence_matching: None,
            prompt: prompt.to_string(),
            prompt_size: prompt.len().try_into().unwrap(),
        };

        return Ok(ok("ok", Some(serde_json::to_value(data).unwrap())));
    }

    if prompt.len() > router.max_prompt_length.try_into().unwrap() || prompt.len() < 1 {
        return Err(bad_request("prompt.length.invalid", None));
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
                return Err(bad_request("prompt.calification.error", None));
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
                        return Err(bad_request("model.not.found", None));
                    }
                };

                let data = ProccesedPrompt {
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
                };

                return Ok(ok("ok", Some(serde_json::to_value(data).unwrap())));
            }

            return Err(bad_request("prompt.calification.error", None));
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
                    return Err(bad_request("model.not.found", None));
                }
            };

            if sentence.exact && sentence.text.to_lowercase() == prompt.to_lowercase() {
                let data = ProccesedPrompt {
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
                };

                return Ok(ok("ok", Some(serde_json::to_value(data).unwrap())));
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
                        return Err(bad_request("sentence.matching.error", None));
                    }
                };

                if index == router.sentences.len() - 1 && !similar {
                    let data = ProccesedPrompt {
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
                    };

                    return Ok(ok("ok", Some(serde_json::to_value(data).unwrap())));
                }

                if !similar {
                    continue;
                }

                let data = ProccesedPrompt {
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
                };

                return Ok(ok("ok", Some(serde_json::to_value(data).unwrap())));
            }
        }
    }

    return Err(bad_request("prompt.calification.error", None));
}

pub async fn get_models_list(
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let data = LLMs::all_models_info();
    Ok(ok("ok", Some(serde_json::to_value(data).unwrap())))
}
