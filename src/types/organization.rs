use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use super::{customer::CustomerID, router::Router};

pub type OrganizationID = String;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelOwner {
    OpenAI,
    Anthropic,
    Coherence,
    OpenSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Legacy,
    Custom,
}

impl From<ModelType> for Bson {
    fn from(model_type: ModelType) -> Self {
        match model_type {
            ModelType::Legacy => Bson::String("legacy".to_string()),
            ModelType::Custom => Bson::String("custom".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelObject {
    pub id: String,
    pub r#type: ModelType,
    pub display_name: String,
    pub registered_by: CustomerID,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Member,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    pub id: CustomerID,
    pub role: MemberRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenScopes {
    Admin,
    ManageModels,
    ManageRouters,
    ManageMembers,
    AccessPromptModelSuggestion,
    AccessCachingService
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
