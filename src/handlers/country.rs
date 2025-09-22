use actix_web::{Responder, Scope, delete, get, post, put, web};

use crate::{AppState, producer};

// ESTRUCTURA PARA EL PAYLOAD DE CREACIÓN DE PAÍS (campos obligatorios)
#[derive(serde::Deserialize, serde::Serialize)]
struct CreateCountryRequest {
    name: String,
    code: String,
    dial_code: String,
}

// ESTRUCTURA PARA EL PAYLOAD DE ACTUALIZACIÓN DE PAÍS (campos opcionales)
#[derive(serde::Deserialize, serde::Serialize)]
struct UpdateCountryRequest {
    name: Option<String>,
    code: Option<String>,
    dial_code: Option<String>,
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

#[put("/{id}")]
async fn update_country_handler(
    _state: web::Data<AppState>,
    _id: web::Path<String>,
    _payload: web::Json<UpdateCountryRequest>,
) -> impl Responder {
    producer::publish_message(
        &_state,
        "updateCountry",
        serde_json::json!({"id": _id.into_inner(), "updateCountryDto":_payload}),
    )
    .await
}

#[delete("/{id}")]
async fn delete_country_handler(
    _state: web::Data<AppState>,
    _id: web::Path<String>,
) -> impl Responder {
    producer::publish_message(&_state, "removeCountry", _id.into_inner().into()).await
}

#[get("/logs")]
async fn get_country_logs_handler(_state: web::Data<AppState>) -> impl Responder {
    producer::publish_message(&_state, "getLogs", "{}".into()).await
}

pub fn countries_scope() -> Scope {
    web::scope("/countries")
        .service(get_countries_handler)
        .service(create_country_handler)
        .service(update_country_handler)
        .service(delete_country_handler)
        .service(get_country_logs_handler)
}
