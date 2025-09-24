use std::{env, sync::LazyLock, time::Duration};

use actix_web::HttpResponse;
use lapin::{BasicProperties, options::BasicPublishOptions};
use tokio::sync::oneshot;
use tracing::info;
use uuid::Uuid;

use crate::AppState;

static QUEUE: LazyLock<String> =
    LazyLock::new(|| env::var("RABBITMQ_QUEUE").expect("RABBITMQ_QUEUE env var not set"));

pub async fn publish_message(
    state: &AppState,
    command: &str,
    payload: serde_json::Value,
) -> HttpResponse {
    let correlation_id = Uuid::new_v4().to_string();
    let data = serde_json::json!({"pattern": {"cmd": command}, "data": payload, "id": correlation_id.clone()});
    let (tx, rx) = oneshot::channel();

    state
        .pending_replies
        .lock()
        .await
        .insert(correlation_id.clone(), tx);

    let props = BasicProperties::default()
        .with_correlation_id(correlation_id.clone().into())
        .with_reply_to(state.reply_queue_name.as_str().into());
    if state
        .amqp_channel
        .basic_publish(
            "",
            QUEUE.as_str(),
            BasicPublishOptions::default(),
            &serde_json::to_vec(&data).unwrap(),
            props,
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    info!(
        "Published RPC request with correlation_id: {}",
        correlation_id
    );

    match tokio::time::timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(response_data)) => {
            match serde_json::from_slice::<serde_json::Value>(&response_data) {
                Ok(json_data) => {
                    HttpResponse::Ok().json(json_data.get("response").unwrap_or(&json_data))
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => {
            state.pending_replies.lock().await.remove(&correlation_id);
            HttpResponse::GatewayTimeout().finish()
        }
    }
}
