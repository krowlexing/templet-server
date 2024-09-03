use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_utils::{unwrap_json, Claim};
use serde::Deserialize;

use crate::db::Db;

use super::tokens::AppClaim;

pub async fn all(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Path(app_id): Path<i32>,
) -> impl IntoResponse {
    let sql = db.operators.for_app(app_id, claim.user_id);

    match sql {
        Ok(operators) => unwrap_json(&operators).into_response(),
        Err(e) => {
            println!("Erro while processing '/operators/all'. Error: {e:?}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorId {
    operator_id: i32,
}

pub async fn create(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Path(app_id): Path<i32>,
    Json(body): Json<OperatorId>,
) -> impl IntoResponse {
    let sql = db.operators.create(app_id, claim.user_id, body.operator_id);
    println!("handle create");
    match sql {
        Ok(operator) => unwrap_json(&operator).into_response(),
        Err(e) => {
            println!("error while creating operator: {e:?}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn delete(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Path((app_id, operator_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let sql = db.operators.delete(app_id, claim.user_id, operator_id);

    match sql {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
