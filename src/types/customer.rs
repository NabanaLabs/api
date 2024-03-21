use crate::types::subscription::Subscription;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use super::organization::OrganizationID;

pub type CustomerID = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericResponse {
    pub message: String,
    pub data: Value,
    pub exit_code: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub address: String,
    pub verified: bool,
    pub main: bool,
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CustomerType {
    PERSONAL,
    BUSINESS,
    DEVELOPER,
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthProviders {
    GOOGLE,
    LEGACY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: CustomerID,
    pub name: String,
    pub class: CustomerType,
    pub emails: Vec<Email>,
    pub auth_provider: AuthProviders,

    // security
    pub password: String, // store the hashed password
    pub backup_security_codes: Vec<String>, // stire hashed backup security codes

    // miscelaneous
    pub preferences: Preferences,
    pub subscription: Subscription,

    // account state
    pub created_at: String,
    pub updated_at: String,
    pub deleted: bool,

    // org
    pub related_orgs: Vec<OrganizationID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicCustomer {
    pub id: CustomerID,
    pub name: String,
    pub class: CustomerType,
    
    pub preferences: Preferences,
    pub subscription: Subscription,

    pub created_at: String,
    pub updated_at: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateSensitiveCustomer {
    pub id: Option<CustomerID>,
    pub name: Option<String>,
    pub class: Option<CustomerType>,
    pub emails: Option<Vec<Email>>,
    pub auth_provider: Option<AuthProviders>,

    // miscelaneous
    pub preferences: Option<Preferences>,
    pub subscription: Option<Subscription>,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted: Option<bool>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub dark_mode: bool,
    pub language: String,
    pub notifications: bool,
}
