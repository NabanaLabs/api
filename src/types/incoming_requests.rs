use serde::{Deserialize, Serialize};

use super::llm_router::{Category, Sentence};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignIn {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRecord {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub class: String,
    pub accepted_terms: bool,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerUpdateName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerUpdatePassword {
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAddEmail {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct FetchCustomerByID {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailQueryParams {
    pub token: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct ProcessPrompt {
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrg {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct EditOrg {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateModel {
    pub id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveModel {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct EditModel {
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRouter {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRouter {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct EditRouter {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,

    pub active: Option<bool>,
    pub deleted: Option<bool>,

    pub max_prompt_length: Option<i32>,

    pub use_single_model: Option<bool>,
    pub model_id: Option<String>,

    pub use_prompt_calification_model: Option<bool>,
    pub prompt_calification_model_id: Option<String>,
    pub prompt_calification_model_categories: Option<Vec<Category>>,

    pub use_sentence_matching_model: Option<bool>,
    pub sentences: Option<Vec<Sentence>>,
}

#[derive(Debug, Deserialize)]
pub struct AddMember {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveMember {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct EditMember {
    pub role: String,
}