use std::sync::Arc;

use crate::{
    storage::mongo::{build_organizations_filter, find_organization, get_organizations_collection, update_organization},
    types::{
        customer::{CustomerID, GenericResponse},
        incoming_requests::{CreateModel, CreateOrg, CreateRouter, EditModel, EditOrg, RemoveModel},
        llm_router::{self, Router},
        llms::LLMs,
        organization::{MemberRole, ModelObject, OrgMember, Organization},
        state::AppState,
    },
    utilities::helpers::{
        bad_request, internal_server_error, ok, payload_analyzer, random_string, unauthorized
    },
};

use axum::{
    extract::rejection::JsonRejection,
    http::{HeaderMap, StatusCode},
    Json,
};
use mongodb::bson::{doc, Bson};

use super::identity::{get_user_session_from_req,  SessionScopes};

pub struct AccessData {
    pub org_id: String,
    pub customer_id: CustomerID,
    pub customer_scopes: Vec<SessionScopes>,
}

pub async fn extract_access_data(
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Result<AccessData, (StatusCode, Json<GenericResponse>)> {
    let session_data = get_user_session_from_req(&headers, &state.redis_connection).await?;
    if !(session_data.scopes.contains(&SessionScopes::TotalAccess) && session_data.scopes.contains(&SessionScopes::ManageOrganizations))
    {
        return Err(unauthorized("not.enough.scopes", None));
    }

    let org_id = match headers.get("OrganizationID") {
        Some(org_id) => match org_id.to_str() {
            Ok(org_id) => org_id,
            Err(_) => return Err(bad_request("OrganizationID.header.required", None)),
        },
        None => return Err(unauthorized("OrganizationID.header.required", None)),
    };

    let access_data = AccessData {
        org_id: org_id.to_string(),
        customer_id: session_data.customer_id,
        customer_scopes: session_data.scopes,
    };

    Ok(access_data)
}

pub async fn get_org(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let filter = build_organizations_filter(&access_data.org_id).await;
    let mut org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member || member.role == MemberRole::Viewer)) {
        return Err(unauthorized("not.org.member", None));
    }

    let member = org.members.iter().find(|member| member.id == access_data.customer_id).unwrap();
    if member.role == MemberRole::Owner {
        return Ok(ok("ok", Some(serde_json::to_value(org).unwrap())));
    }

    org.access_tokens = vec![];

    return Ok(ok("ok", Some(serde_json::to_value(org).unwrap())));
}

pub async fn create_org(
    headers: HeaderMap,
    payload_result: Result<Json<CreateOrg>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let session_data = get_user_session_from_req(&headers, &state.redis_connection).await?;
    if !(session_data.scopes.contains(&SessionScopes::TotalAccess) && session_data.scopes.contains(&SessionScopes::ManageOrganizations))
    {
        return Err(unauthorized("not.enough.scopes", None));
    }

    let payload = payload_analyzer(payload_result)?;
    if payload.name.len() < 3 || payload.name.len() > 30 {
        return Err(bad_request("org.name.length.invalid", None));
    }

    // create
    let id = random_string(32).await;
    let org = Organization {
        id: id.clone(),
        name: payload.name.clone(),
        models: vec![],
        routers: vec![],
        members: vec![OrgMember {
            id: session_data.customer_id,
            role: MemberRole::Owner,
        }],
        access_tokens: vec![],
        deleted: false,
    };

    let collection = get_organizations_collection(&state.mongo_db).await;
    match collection.insert_one(org.clone(), None).await {
        Ok(_) => (),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }

    return Ok(ok("ok", Some(serde_json::to_value(org).unwrap())));
}

pub async fn edit_org(
    headers: HeaderMap,
    payload_result: Result<Json<EditOrg>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let payload = payload_analyzer(payload_result)?;
    if payload.name.len() < 3 || payload.name.len() > 30 {
        return Err(bad_request("org.name.length.invalid", None));
    }

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }

    if org.name == payload.name {
        return Ok(ok("ok", None));
    }

    let update = doc! {"$set": {
            "name": &payload.name,
        }
    };

    let filter = build_organizations_filter(&access_data.org_id).await;
    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", None));
}

pub async fn delete_org(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && member.role == MemberRole::Owner) {
        return Err(unauthorized("not.org.owner", None));
    }

    let update = doc! {"$set": {
            "deleted": true,
        }
    };

    let filter = build_organizations_filter(&access_data.org_id).await;
    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", None));
}

pub async fn get_models(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member || member.role == MemberRole::Viewer)) {
        return Err(unauthorized("not.org.member", None));
    }

    return Ok(ok("ok", Some(serde_json::to_value(org.models).unwrap())));
}

pub async fn create_model_org(
    headers: HeaderMap,
    payload_result: Result<Json<CreateModel>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let payload = payload_analyzer(payload_result)?;

    if payload.id.len() < 1 || payload.id.len() > 256 {
        return Err(bad_request("model.id.length.invalid", None));
    }

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }

    let all_models_list = LLMs::all_models_info();
    let model = match all_models_list.iter().find(|model| model.model.clone() == payload.id) {
        Some(model) => model,
        None => return Err(bad_request("model.not.found", None)),
    };

    if payload.display_name.len() < 1 || payload.display_name.len() > 32 {
        return Err(bad_request("model.display_name.length.invalid", None));
    }
    
    if payload.description.len() < 1 || payload.description.len() > 128 {
        return Err(bad_request("model.description.length.invalid", None));
    }

    let model_object = ModelObject {
        id: model.model.clone(),
        display_name: payload.display_name.clone(),
        description: payload.description.clone(),
        company: model.company.clone(),
        registered_by: access_data.customer_id,
    };

    let model_object_bson = doc! {
        "id": model_object.id.clone(),
        "display_name": model_object.display_name.clone(),
        "description": model_object.description.clone(),
        "company": model_object.company.clone(),
        "registered_by": model_object.registered_by.clone(),
    };

    let update = doc! {"$push": {
            "models": model_object_bson,
        }
    };

    let filter = build_organizations_filter(&access_data.org_id).await;
    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", Some(serde_json::to_value(model_object).unwrap())));
}

pub async fn delete_model_org(
    headers: HeaderMap,
    payload_result: Result<Json<RemoveModel>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let payload = payload_analyzer(payload_result)?;

    if payload.id.len() < 1 || payload.id.len() > 256 {
        return Err(bad_request("model.id.length.invalid", None));
    }

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }

    let all_models_list = LLMs::all_models_info();
    let model = match all_models_list.iter().find(|model| model.model.clone() == payload.id) {
        Some(model) => model,
        None => return Err(bad_request("model.not.found", None)),
    };

    let update = doc! {"$pull": {
            "models": doc!{"id": model.model.clone()},
        }
    };

    let filter = build_organizations_filter(&access_data.org_id).await;
    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", None));
}

pub async fn edit_model_org(
    headers: HeaderMap,
    payload_result: Result<Json<EditModel>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let payload = payload_analyzer(payload_result)?;

    if payload.id.len() < 1 || payload.id.len() > 256 {
        return Err(bad_request("model.id.length.invalid", None));
    }

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }

    let all_models_list = LLMs::all_models_info();
    let model = match all_models_list.iter().find(|model| model.model == payload.id) {
        Some(model) => model,
        None => return Err(bad_request("model.not.found", None)),
    };

    let filter = doc! { 
        "id": org.id, 
        "models.id": model.model.clone()
    };

    let update = doc! {
        "$set": { 
            "models.$.display_name": &payload.display_name,
            "models.$.description": &payload.description,
        }
    };

    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", None));
}

pub async fn get_routers(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member || member.role == MemberRole::Viewer)) {
        return Err(unauthorized("not.org.member", None));
    }

    return Ok(ok("ok", Some(serde_json::to_value(org.routers).unwrap())));
}

pub async fn create_router_org(
    headers: HeaderMap,
    payload_result: Result<Json<CreateRouter>, JsonRejection>,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let payload = payload_analyzer(payload_result)?;

    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }

    if payload.name.len() < 1 || payload.name.len() > 32 {
        return Err(bad_request("router.name.length.invalid", None));
    }

    if payload.description.len() < 1 || payload.description.len() > 128 {
        return Err(bad_request("router.description.length.invalid", None));
    }

    let id = random_string(32).await;
    let router = Router {
        id: id.clone(),
        name: payload.name.clone(),
        description: payload.description.clone(),

        active: true,
        deleted: false,

        max_prompt_length: 512,

        use_single_model: false,
        model_id: "".to_string(),

        use_prompt_calification_model: false,
        prompt_calification_model_version: "".to_string(),
        prompt_calification_model_categories: vec![],

        use_sentence_matching: false,
        sentences: vec![],
    };

    let update = doc! {"$push": {
            "routers": <llm_router::Router as Into<Bson>>::into(router.clone()),
        }
    };

    let filter = build_organizations_filter(&access_data.org_id).await;
    update_organization(&state.mongo_db, filter, update).await?;

    return Ok(ok("ok", Some(serde_json::to_value(router).unwrap())));
}

// org access template
#[allow(dead_code)]
pub async fn org_access_template_org(
    headers: HeaderMap,
    state: Arc<AppState>,
) -> Result<(StatusCode, Json<GenericResponse>), (StatusCode, Json<GenericResponse>)> {
    let access_data = extract_access_data(&headers, &state).await?;
    let filter = build_organizations_filter(&access_data.org_id).await;
    let org = find_organization(&state.mongo_db, filter).await?;

    if !org.members.iter().any(|member| member.id == access_data.customer_id && (member.role == MemberRole::Owner || member.role == MemberRole::Member)) {
        return Err(unauthorized("not.org.member", None));
    }


    return Ok(ok("ok", None));
}