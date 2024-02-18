use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{customer::CustomerID, llm_router::Router};

pub type OrganizationID = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelOwner {
    OpenAI,
    Anthropic,
    Coherence,
    OpenSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelObject {
    pub id: String,
    pub model_name: String,
    pub display_name: String,
    pub description: String,
    pub owner: ModelOwner,
    pub registered_by: CustomerID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    pub id: CustomerID,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessTokenScopes {
    Admin,
    ManageModels,
    ManageRouters,
    ManageMembers,
    AccessPromptModelSuggestion,
    AccessCachingService
}

impl AccessTokenScopes {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "manage.models" => Some(Self::ManageModels),
            "manage.routers" => Some(Self::ManageRouters),
            "manage.members" => Some(Self::ManageMembers),
            "access.prompt.model.suggestion" => Some(Self::AccessPromptModelSuggestion),
            "access.caching.service" => Some(Self::AccessCachingService),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Admin => String::from("admin"),
            Self::ManageModels => String::from("manage.models"),
            Self::ManageRouters => String::from("manage.routers"),
            Self::ManageMembers => String::from("manage.members"),
            Self::AccessPromptModelSuggestion => String::from("access.prompt.model.suggestion"),
            Self::AccessCachingService => String::from("access.caching.service"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub created_by: CustomerID,
    pub created_at: String,
    pub token: String,
    pub scopes: Vec<AccessTokenScopes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationID,
    pub name: String,
    pub models: Vec<ModelObject>,
    pub routers: Vec<Router>,
    pub members: Vec<OrgMember>,
    pub access_tokens: Vec<AccessToken>,
    pub deleted: bool,
}