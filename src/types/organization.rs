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
pub struct Organization {
    pub id: OrganizationID,
    pub name: String,
    pub models: Vec<ModelObject>,
    pub routers: Vec<Router>,
    pub members: Vec<OrgMember>,
    pub deleted: bool,
}