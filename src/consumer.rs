use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use lapin::{options::BasicConsumeOptions, types::FieldTable};
use tokio::sync::{Mutex, oneshot};
use tracing::{info, warn};

pub async fn response_listener_task(
    channel: Arc<lapin::Channel>,
    reply_queue_name: Arc<String>,
    pending_replies: Arc<Mutex<HashMap<String, oneshot::Sender<Vec<u8>>>>>,
) {
    let mut consumer = channel
        .basic_consume(
            &reply_queue_name,
            "",
            BasicConsumeOptions {
                no_ack: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!("Response listener started on queue: {}", reply_queue_name);

    // Bucle para procesar las respuestas que llegan
    while let Some(Ok(delivery)) = consumer.next().await {
        if let Some(correlation_id) = delivery.properties.correlation_id() {
            let correlation_id = correlation_id.to_string();

            // Buscar el `Sender` para esta respuesta en el mapa
            if let Some(sender) = pending_replies.lock().await.remove(&correlation_id) {
                // Enviar la data de vuelta al handler que está esperando
                if sender.send(delivery.data).is_err() {
                    warn!(
                        "Failed to send response for correlation_id: {}",
                        correlation_id
                    );
                }
            }
        }
    }
}
