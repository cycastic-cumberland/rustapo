use std::collections::HashMap;
use actix_web::{HttpResponse, post, ResponseError, Scope};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::evaluator::evaluator::{IRustapoEvaluator, IRustapoWrappedInstance};
use crate::evaluator::lua_evaluator::LuaEvaluator;

#[derive(Debug, Serialize)]
pub struct ResultReturn<T> {
    result: T
}

#[derive(Debug, Deserialize)]
pub struct BooleanEvaluationRequest {
    expression: String,
    f64: Option<HashMap<String, f64>>,
    strings: Option<HashMap<String, String>>
}

#[derive(Debug, Display, Serialize)]
pub struct ErrorReturn {
    error: String
}

impl ResponseError for ErrorReturn {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(match serde_json::to_string(self){
                Ok(v) => { v }
                Err(_) => { self.error.clone() }
            })
    }
}

#[post("/eval")]
pub async fn evaluate_boolean(evaluator: Data<LuaEvaluator>, payload: Json<BooleanEvaluationRequest>) -> Result<Json<ResultReturn<bool>>, ErrorReturn> {
    let instance = evaluator.capture();
    if let Some(s) = &payload.strings {
        for (key, value) in s {
            if let Err(e) = instance.declare_string(key, value.clone()) {
                return Err(ErrorReturn{
                    error: e.to_string()
                })
            }
        }
    }
    if let Some(s) = &payload.f64 {
        for (key, value) in s {
            if let Err(e) = instance.declare_f64(key, *value) {
                return Err(ErrorReturn{
                    error: e.to_string()
                })
            }
        }
    }
    match instance.evaluate_boolean(&payload.expression).get_result() {
        Ok(v) => {
            Ok(Json(ResultReturn{
                result: v
            }))
        }
        Err(e) => {
            Err(ErrorReturn{
                error: e.to_string()
            })
        }
    }
}

pub fn map() -> Scope {
    actix_web::web::scope("")
        .service(evaluate_boolean)
}