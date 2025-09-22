use actix_web::{Responder, Scope, get, post, web};

use crate::{AppState, producer};

// ESTRUCTURA PARA EL PAYLOAD DE CREACIÓN DE PAÍS
#[derive(serde::Deserialize, serde::Serialize)]
struct CreateCountryRequest {
    name: String,
    code: String,
    dial_code: String,
}

#[post("")]
async fn create_country_handler(
    _state: web::Data<AppState>, // Recibimos el estado con web::Data
    _payload: web::Json<CreateCountryRequest>,
) -> impl Responder {
    producer::publish_message(
        &_state,
        "createCountry",
        serde_json::to_value(_payload).unwrap(),
    )
    .await
}

// EL HANDLER ADAPTADO PARA ACTIX WEB
#[get("")]
async fn get_countries_handler(
    _state: web::Data<AppState>, // Recibimos el estado con web::Data
) -> impl Responder {
    producer::publish_message(&_state, "findByCriteria", "{}".into()).await
}

pub fn countries_scope() -> Scope {
    web::scope("/countries")
        .service(get_countries_handler)
        .service(create_country_handler)
}
