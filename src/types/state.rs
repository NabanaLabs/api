use std::sync::{Arc, Mutex};

use diesel::{r2d2::ConnectionManager, PgConnection};
use r2d2::Pool;

use mongodb::{Client as MongoClient, Database};
use redis::Client as RedisClient;
use rust_bert::pipelines::{sentence_embeddings::SentenceEmbeddingsModel, zero_shot_classification::ZeroShotClassificationModel};

use super::lemonsqueezy::Products;

#[derive(Clone)]
pub struct MasterEmailEntity {
    pub email: String,
    pub name: String,
}

#[derive(Clone)]
pub struct EmailProviderSettings {
    pub email_verification_template_id: u32,
}

#[derive(Clone)]
pub struct GoogleAuth {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

#[derive(Clone)]
pub struct PromptClassificationModel {
    pub model: Arc<Box<Mutex<ZeroShotClassificationModel>>>,
    pub name: String,
    pub url: String,
}

#[derive(Clone)]
pub struct LLMResources {
    pub prompt_classification_model: PromptClassificationModel,
    pub embedding_model: Arc<Box<Mutex<SentenceEmbeddingsModel>>>,
}

#[derive(Clone)]
pub struct AppState {
    pub api_url: String,
    pub api_tokens_expiration_time: i64,

    pub mongodb_client: MongoClient,
    pub mongo_db: Database,

    pub redis_connection: RedisClient,
    pub postgres_conn: Option<Pool<ConnectionManager<PgConnection>>>,

    pub lemonsqueezy_webhook_signature_key: String,
    pub products: Products,

    pub enabled_email_integration: bool,
    pub master_email_entity: MasterEmailEntity,
    pub email_provider_settings: EmailProviderSettings,

    pub google_auth: GoogleAuth,
    pub llm_resources: LLMResources,
}