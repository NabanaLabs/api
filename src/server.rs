use crate::{
    routers::{
        core::get_core_router, customers::get_customers_router, identity::get_identity_router, org::get_org_router, webhooks::get_webhooks_router
    }, types::{lemonsqueezy::Products, state::{AppState, EmailProviderSettings, GoogleAuth, MasterEmailEntity}}, utilities::helpers::fallback
};
use axum::{
    http::Method,
    routing::get,
    Router,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use mongodb::Client as MongoClient;
use r2d2::Pool;
use redis::Client as RedisClient;
use rust_bert::{pipelines::{common::{ModelResource, ModelType}, sentence_embeddings::{SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType}, zero_shot_classification::{ZeroShotClassificationConfig, ZeroShotClassificationModel}}, resources::RemoteResource, RustBertError};
use tokio::task;
use std::{env, sync::{Arc, Mutex}, time::Duration};

use tower_http::timeout::TimeoutLayer;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};

use log::info;

pub async fn init(mongodb_client: MongoClient, redis_connection: RedisClient, postgres_conn: Option<Pool<ConnectionManager<PgConnection>>>) {
    let app_state = set_app_state(mongodb_client, redis_connection, postgres_conn).await;

    // /api/org
    let org = get_org_router(app_state.clone()).await;
    // /api/customers
    let customers = get_customers_router(app_state.clone()).await;
    info!("Customers router loaded");
    // /api/identity
    let identity = get_identity_router(app_state.clone()).await;
    info!("Identity router loaded");
    // /api/webhooks
    let webhooks = get_webhooks_router(app_state.clone()).await;
    info!("Webhooks router loaded");
    // /api/llm/routers
    let core = get_core_router(app_state.clone()).await;
    info!("Core router loaded");
    // /api
    let api = Router::new()
        .nest("/org", org)
        .nest("/customers", customers)
        .nest("/identity", identity)
        .nest("/webhooks", webhooks)
        .nest("/core", core);

    info!("API router loaded");

    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_origin(Any);

    let app = Router::new()
        .route("/service/health", get(|| async { "OK" }))
        .nest("/api", api)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(10)),)
        .fallback(fallback)
        .with_state(app_state);

    let host = env::var("HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let address = format!("{}:{}", host, port);

    info!("Starting server on {}", address);

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(e) => panic!("Error binding to address: {}", e),
    };

    match axum::serve(listener, app).await {
        Ok(_) => info!("Server started"),
        Err(e) => panic!("Error starting server: {}", e),
    };
}

pub async fn set_app_state(mongodb_client: MongoClient, redis_connection: RedisClient, postgres_conn: Option<Pool<ConnectionManager<PgConnection>>>) -> Arc<AppState> {
    let api_url = match env::var("API_URL") {
        Ok(url) => url,
        Err(_) => panic!("api_url not found"),
    };

    let mongo_db = match env::var("MONGO_DB_NAME") {
        Ok(db) => db,
        Err(_) => panic!("mongo_db_name not found"),
    };

    let mongo_db = mongodb_client.database(&mongo_db);
    let lemonsqueezy_webhook_signature_key = match env::var("LEMONSQUEEZY_WEBHOOK_SIGNATURE_KEY") {
        Ok(uri) => uri,
        Err(_) => String::from("lemonsqueezy_webhook_signature_key not found"),
    };

    let pro_product_id = match env::var("PRO_PRODUCT_ID") {
        Ok(id) => match id.parse::<i64>() {
            Ok(id) => id,
            Err(_) => panic!("pro_product_id must be a number"),
        },
        Err(_) => panic!("pro_product_id not found"),
    };

    let pro_monthly_variant_id = match env::var("PRO_MONTHLY_VARIANT_ID") {
        Ok(id) => match id.parse::<i64>() {
            Ok(id) => id,
            Err(_) => panic!("pro_monthly_variant_id must be a number"),
        },
        Err(_) => panic!("pro_monthly_variant_id not found"),
    };

    let pro_annually_variant_id = match env::var("PRO_ANNUALLY_VARIANT_ID") {
        Ok(id) => match id.parse::<i64>() {
            Ok(id) => id,
            Err(_) => panic!("pro_annually_variant_id must be a number"),
        },
        Err(_) => panic!("pro_annually_variant_id not found"),
    };

    let products = Products {
        pro_product_id,
        pro_monthly_variant_id,
        pro_annually_variant_id,
    };

    let enabled_email_integration = match std::env::var("ENABLE_EMAIL_INTEGRATION").expect("ENABLE_EMAIL_INTEGRATION must be set").parse::<bool>() {
        Ok(val) => val,
        Err(_) => panic!("ENABLE_EMAIL_INTEGRATION must be a boolean"),
    };

    let api_tokens_expiration_time = match std::env::var("API_TOKENS_EXPIRATION_TIME").expect("API_TOKENS_EXPIRATION_TIME must be set").parse::<i64>() {
        Ok(val) => val,
        Err(_) => panic!("API_TOKENS_EXPIRATION_TIME must be a number"),
    };

    let master_email_address = env::var("BREVO_MASTER_EMAIL_ADDRESS");
    let master_name = env::var("BREVO_MASTER_NAME");

    let master_email_entity = MasterEmailEntity {
        email: match master_email_address {
            Ok(email) => email,
            Err(_) => panic!("BREVO_MASTER_EMAIL_ADDRESS not found"),
        },
        name: match master_name {
            Ok(name) => name,
            Err(_) => panic!("BREVO_MASTER_NAME not found"),
        },
    };

    let email_verification_template_id = match env::var("BREVO_EMAIL_VERIFY_TEMPLATE_ID") {
        Ok(id) => match id.parse::<u32>() {
            Ok(id) => id,
            Err(_) => panic!("BREVO_EMAIL_VERIFY_TEMPLATE_ID must be a number"),
        },
        Err(_) => panic!("BREVO_EMAIL_VERIFY_TEMPLATE_ID not found"),
    };

    let email_provider_settings = EmailProviderSettings {
        email_verification_template_id,
    };

    let google_oauth_redirect_endpoints = match env::var("GOOGLE_OAUTH_CLIENT_REDIRECT_ENDPOINT") {
        Ok(url) => url,
        Err(_) => panic!("GOOGLE_OAUTH_CLIENT_REDIRECT_ENDPOINT not found"),
    };

    let google_oauth_redirect_url = format!("https://{}{}", api_url, google_oauth_redirect_endpoints);

    let google_auth = GoogleAuth {
        client_id: match env::var("GOOGLE_OAUTH_CLIENT_ID") {
            Ok(id) => id,
            Err(_) => panic!(" GOOGLE_OAUTH_CLIENT_ID not found"),
        },
        client_secret: match env::var("GOOGLE_OAUTH_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_) => panic!("GOOGLE_OAUTH_CLIENT_SECRET not found"),
        },
        redirect_url: google_oauth_redirect_url,
    };

    let prompt_classification_model_name = match env::var("PROMPT_CLASSIFICATION_MODEL_NAME") {
        Ok(name) => name,
        Err(_) => panic!("PROMPT_CLASSIFICATION_MODEL_NAME not found"),
    };

    let prompt_classification_model_url = match env::var("PROMPT_CLASSIFICATION_MODEL_URL") {
        Ok(url) => url,
        Err(_) => panic!("PROMPT_CLASSIFICATION_MODEL_URL not found"),
    };

    let zero_shot_prompt_classification_model = match zero_shot_create_prompt_classify_model(&prompt_classification_model_name, &prompt_classification_model_url).await {
        Ok(model) => model,
        Err(e) => panic!("Error creating prompt classification model: {}", e),
    };

    let zero_shot_safe_prompt_classification_model = Arc::new(Box::new(Mutex::new(zero_shot_prompt_classification_model)));

    let embedding_model_result = match task::spawn_blocking(move || {
        SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2).create_model()
    }).await.map_err(|e| RustBertError::TchError(e.to_string())) {
        Ok(model) => model,
        Err(e) => panic!("Error creating sentence embedding model: {}", e),
    };

    let embedding_model = match embedding_model_result {
        Ok(model) => model,
        Err(e) => panic!("Error creating sentence embedding model: {}", e),
    };

    let safe_embedding_model = Arc::new(Box::new(Mutex::new(embedding_model)));

    let llm_resources = crate::types::state::LLMResources {
        prompt_classification_model: crate::types::state::PromptClassificationModel {
            model: zero_shot_safe_prompt_classification_model,
            name: prompt_classification_model_name,
            url: prompt_classification_model_url,
        },
        embedding_model: safe_embedding_model,
    };
    let app_state = Arc::new(AppState {
        mongodb_client,
        redis_connection,
        postgres_conn,
        mongo_db,
        lemonsqueezy_webhook_signature_key,
        products,
        enabled_email_integration,
        api_tokens_expiration_time,
        api_url,
        master_email_entity,
        email_provider_settings,
        google_auth,
        llm_resources,
    });

    return app_state;
}

// test function to test with another different model
pub async fn zero_shot_create_prompt_classify_model(name: &String, url: &String) -> Result<ZeroShotClassificationModel, RustBertError> {
    info!("Loading Prompt Classification Model: {} from: {}", name, url);                 
    let config_resource = Box::new(RemoteResource::from_pretrained((
        name,
        &format!("{}/resolve/main/config.json", url).to_string()
    )));

    let vocab_resource = Box::new(RemoteResource::from_pretrained((
        name,
        &format!("{}/resolve/main/vocab.json", url).to_string()
    )));

    let merges_resource = Box::new(RemoteResource::from_pretrained((
        name,
        &format!("{}/resolve/main/merges.txt", url).to_string()
    )));
    
    let model_resource = ModelResource::Torch(Box::new(RemoteResource::from_pretrained((
        name,
        &format!("{}/resolve/main/rust_model.ot", url).to_string()
    ))));

    let config = ZeroShotClassificationConfig {
        model_type: ModelType::Bart,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: Some(merges_resource),
        ..Default::default()
    };

    let model_result = task::spawn_blocking(move || {
        ZeroShotClassificationModel::new(config)
    }).await.map_err(|e| RustBertError::TchError(e.to_string()))?;

    return model_result;
}