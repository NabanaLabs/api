use std::sync::Arc;

use axum::{http::StatusCode, Json};
use mongodb::bson::{doc, to_bson, Bson};

use crate::{
    storage::mongo::{build_customer_filter, find_customer, update_customer}, types::{
        customer::GenericResponse, lemonsqueezy::SubscriptionEvent, state::AppState, subscription::{Slug, Subscription, SubscriptionFrequencyClass, SubscriptionHistoryLog}
    }, utilities::helpers::{add_subscription_history_log_and_to_bson, bad_request, internal_server_error, random_string}
};

pub async fn subscription_created(
    event: SubscriptionEvent,
    state: Arc<AppState>,
) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let customer_id = event.meta.custom_data.unwrap().customer_id;
    let filter = build_customer_filter(customer_id.as_str(), event.data.attributes.user_email.as_str()).await;
    let customer = find_customer(&state.mongo_db, filter.clone()).await?;

    let frequency: SubscriptionFrequencyClass;
    if event.data.attributes.variant_id == state.products.pro_monthly_variant_id {
        frequency = SubscriptionFrequencyClass::MONTHLY;
    } else if event.data.attributes.variant_id == state.products.pro_annually_variant_id {
        frequency = SubscriptionFrequencyClass::ANNUALLY;
    } else {
        return Err(bad_request("subscription.variant.not.found", None));
    }

    let subscription_id = random_string(15).await;
    let mut history_logs = customer.subscription.history_logs.clone();
    history_logs.push(SubscriptionHistoryLog {
        event: event.meta.event_name,
        date: event.data.attributes.updated_at.clone(),
    });

    let mut slug = Slug::FREE.to_string();
    if event.data.attributes.product_id == state.products.pro_product_id {
        slug = Slug::PRO.to_string();
    }

    let ends_at = match event.data.attributes.ends_at {
        Some(ends_at) => ends_at,
        None => "".to_string(),
    };
    
    let update_subscription = Subscription {
        id: subscription_id,
        product_id: event.data.attributes.product_id,
        variant_id: event.data.attributes.variant_id,
        slug,
        frequency,
        status: event.data.attributes.status,
        created_at: customer.created_at,
        updated_at: event.data.attributes.updated_at,
        starts_at: event.data.attributes.created_at,
        ends_at,
        renews_at: event.data.attributes.renews_at,
        history_logs,
    };

    let update_subscription = match to_bson(&update_subscription) {
        Ok(Bson::Document(document)) => document,
        _ => {
            return Err(internal_server_error("database.error", None));
        }
    };

    let update = doc! {
        "$set": doc!{
            "subscription": update_subscription
        },
    };

    match update_customer(&state.mongo_db, filter, update).await {
        Ok(_) => Ok(()),
        Err(_) => {
            return Err(internal_server_error("database.error", None));
        }
    }
}

pub async fn subscription_updated(
    event: SubscriptionEvent,
    state: Arc<AppState>,
) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let customer_id = event.meta.custom_data.unwrap().customer_id;
    let filter = build_customer_filter(customer_id.as_str(), event.data.attributes.user_email.as_str()).await;
    let customer = find_customer(&state.mongo_db, filter.clone()).await?;

    let bson_history_logs = add_subscription_history_log_and_to_bson(customer.subscription.history_logs, SubscriptionHistoryLog {
        event: event.meta.event_name,
        date: event.data.attributes.updated_at.clone(),
    }).await;

    let update = doc! {
        "$set": doc!{
            "subscription.variant_id": event.data.attributes.variant_id as i64,
            "subscription.status": event.data.attributes.status,
            "subscription.updated_at": event.data.attributes.updated_at,
            "subscription.history_logs": bson_history_logs,
        },
    };

    update_customer(&state.mongo_db, filter, update).await?;
    Ok(())
}

// ready
pub async fn subscription_update_status(
    event: SubscriptionEvent,
    state: Arc<AppState>,
) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let customer_id = event.meta.custom_data.unwrap().customer_id;
    let filter = build_customer_filter(customer_id.as_str(), event.data.attributes.user_email.as_str()).await;
    let customer = find_customer(&state.mongo_db, filter.clone()).await?;

    let bson_history_logs = add_subscription_history_log_and_to_bson(customer.subscription.history_logs, SubscriptionHistoryLog {
        event: event.meta.event_name,
        date: event.data.attributes.updated_at.clone(),
    }).await;

    let update = doc! {
        "$set": doc!{
            "subscription.status": event.data.attributes.status.clone(),
            "subscription.updated_at": event.data.attributes.updated_at,
            "subscription.history_logs": bson_history_logs,
        },
    };

    update_customer(&state.mongo_db, filter, update).await?;
    Ok(())
}

pub async fn subscription_update_history_logs(
    event: SubscriptionEvent,
    state: Arc<AppState>,
) -> Result<(), (StatusCode, Json<GenericResponse>)> {
    let customer_id = event.meta.custom_data.unwrap().customer_id;
    let filter = build_customer_filter(customer_id.as_str(), event.data.attributes.user_email.as_str()).await;
    let customer = find_customer(&state.mongo_db, filter.clone()).await?;

    let bson_history_logs = add_subscription_history_log_and_to_bson(customer.subscription.history_logs, SubscriptionHistoryLog {
        event: event.meta.event_name,
        date: event.data.attributes.updated_at.clone(),
    }).await;

    let update = doc!  {
        "$set": doc!{
            "subscription.updated_at": event.data.attributes.updated_at,
            "subscription.history_logs": bson_history_logs,
        },
    };

    update_customer(&state.mongo_db, filter, update).await?;
    Ok(())
}
