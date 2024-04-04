use axum::{Json, http::StatusCode};
use log::error;
use mongodb::{
    bson::{doc, Document}, options::ClientOptions, options::ServerApi, options::ServerApiVersion, Client, Database, Collection,
};
use serde_json::json;

use std::env;

use crate::{types::{customer::{Customer, GenericResponse}, organization::Organization}, utilities::helpers::{internal_server_error, not_found}};

pub async fn init_connection() -> mongodb::error::Result<Client> {
    let uri = match env::var("MONGO_URI") {
        Ok(uri) => uri,
        Err(_) => String::from("mongo_uri not found"),
    };

    let mut client_options = ClientOptions::parse(&uri).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;

    Ok(client)
}

pub async fn build_customer_filter(id: &str, email: &str) -> Document {
    let customer_filter = doc! {"$or": [
        {"id": id},
        {
            "emails": {
                "$elemMatch": {
                    "address": email,
                }
            }
        }
    ]};

    return customer_filter
}

pub async fn build_organizations_filter(id: &str) -> Document {
    let customer_filter = doc! {"$or": [
        {"id": id},
    ]};

    return customer_filter
}

pub async fn get_customers_collection(db: &Database) -> Collection<Customer> {
    return db.collection("customers");
}

pub async fn get_organizations_collection(db: &Database) -> Collection<Organization> {
    return db.collection("organizations");
}

pub async fn find_customer(db: &Database, filter: Document) -> Result<Customer, (StatusCode, Json<GenericResponse>)> {
    let collection = get_customers_collection(db).await;
    match collection.find_one(filter, None).await {
        Ok(customer) => match customer {
            Some(customer) => Ok(customer),
            None => return Err(not_found("customer.not.found", None)),
        },
        Err(_) => return Err(not_found("customer.not.found", None)),
    }
}

pub async fn find_organization(db: &Database, filter: Document) -> Result<Organization, (StatusCode, Json<GenericResponse>)> {
    let collection = get_organizations_collection(db).await;
    match collection.find_one(filter, None).await {
        Ok(org) => {
            match org {
                Some(org) => return Ok(org),
                None => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(GenericResponse {
                            message: String::from("org.not.found"),
                            data: json!({}),
                            exit_code: 1,
                        }),
                    ));
                },
            }
        },
        Err(e) => {
            error!("error fetching organization: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenericResponse {
                    message: String::from("error fetching organzation"),
                    data: json!({}),
                    exit_code: 1,
                }),
            ));
        },
    }
}

pub async fn update_customer(db: &Database, filter: Document, update: Document) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let collection = get_customers_collection(db).await;
    match collection.update_one(filter, update, None).await {
        Ok(_) => Ok(()),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }
}

pub async fn update_organization(db: &Database, filter: Document, update: Document) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let collection = get_organizations_collection(db).await;
    match collection.update_one(filter, update, None).await {
        Ok(_) => Ok(()),
        Err(_) => return Err(internal_server_error("database.error", None)),
    }
}