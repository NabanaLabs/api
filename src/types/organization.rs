use serde::{Deserialize, Serialize};

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
    pub display_name: String,
    pub description: String,
    pub company: Option<String>,
    pub registered_by: CustomerID,
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
