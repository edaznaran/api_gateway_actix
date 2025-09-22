use actix_web::{App, HttpServer, Responder, web};
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use std::sync::Arc;
use std::{collections::HashMap, env};
use tokio::sync::{Mutex, oneshot};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
mod consumer;
mod producer;

// ESTA ESTRUCTURA NO CAMBIA
#[derive(Clone)]
struct AppState {
    amqp_channel: Arc<lapin::Channel>,
    reply_queue_name: Arc<String>,
    pending_replies: Arc<Mutex<HashMap<String, oneshot::Sender<Vec<u8>>>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");
    // Inicializar el logger
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // connect to RabbitMQ server
    let host = env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST env var not set");
    let port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".into());
    let user = env::var("RABBITMQ_USER").expect("RABBITMQ_USER env var not set");
    let password = env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD env var not set");
    let vhost = env::var("RABBITMQ_VHOST").expect("RABBITMQ_VHOST env var not set");

    let conn = Connection::connect(
        &format!("amqp://{}:{}@{}:{}/{}", user, password, host, port, vhost), // rabbitMQ connection string
        ConnectionProperties::default(), // default connection properties
    )
    .await
    .expect("Failed to connect to RabbitMQ");
    let channel = Arc::new(conn.create_channel().await.unwrap());

    let reply_queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();
    let reply_queue_name = Arc::new(reply_queue.name().to_string());

    let pending_replies = Arc::new(Mutex::new(HashMap::new()));

    // Lanzar la tarea que escucha en la cola de respuesta
    tokio::spawn(consumer::response_listener_task(
        channel.clone(),
        reply_queue_name.clone(),
        pending_replies.clone(),
    ));

    // Crear el estado y envolverlo en web::Data para Actix
    let app_state = AppState {
        amqp_channel: channel,
        reply_queue_name,
        pending_replies,
    };
    let shared_data = web::Data::new(app_state);

    info!("Actix Web server starting on http://127.0.0.1:8080");

    // --- Configuración del Servidor Actix Web ---
    HttpServer::new(move || {
        App::new()
            // Registrar el estado para que esté disponible en los handlers
            .app_data(shared_data.clone())
            // Definir la ruta y el método
            .route("/countries", web::get().to(get_countries_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// ESTRUCTURA PARA EL PAYLOAD DE CREACIÓN DE PAÍS
#[derive(serde::Deserialize, serde::Serialize)]
struct CreateCountryRequest {
    name: String,
    code: String,
    dial_code: String,
}

// EL HANDLER ADAPTADO PARA ACTIX WEB
async fn get_countries_handler(
    state: web::Data<AppState>, // Recibimos el estado con web::Data
) -> impl Responder {
    /*
       let correlation_id = Uuid::new_v4().to_string();
       let (tx, rx) = oneshot::channel();

       let work_queue = env::var("RABBITMQ_QUEUE").expect("RABBITMQ_QUEUE env var not set");
       state
           .pending_replies
           .lock()
           .await
           .insert(correlation_id.clone(), tx);

       let props = BasicProperties::default()
           .with_correlation_id(correlation_id.clone().into())
           .with_reply_to(state.reply_queue_name.as_str().into());
    */
    let payload = serde_json::json!({"pattern": {"cmd": "findByCriteria"}, "data":{}});
    producer::publish_message(&state, payload).await
}
