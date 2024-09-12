use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_utils::{unwrap_json, Claim};
use serde::{Deserialize, Serialize};

use crate::db::{brokers::NewBroker, Db};

use super::tokens::AppClaim;

pub async fn all(
    State(db): State<Db>,
    Path(app_id): Path<i32>,
    Claim(claim): AppClaim,
) -> impl IntoResponse {
    let users = db.brokers.for_app(app_id, claim.user_id);

    match users {
        Ok(users) => unwrap_json(&users).into_response(),
        Err(e) => {
            println!("handlers::app_users::all - {e:?}");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUserBody {
    user_id: i32,
}

pub async fn create(
    State(db): State<Db>,
    Path(app_id): Path<i32>,
    Claim(claim): AppClaim,
    Json(body): Json<NewBroker>,
) -> impl IntoResponse {
    let result = db.brokers.create(app_id, claim.user_id, body);

    match result {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("handlers::app_users::create - {e:?}");
            StatusCode::BAD_REQUEST
        }
    }
}

pub async fn delete(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Path((app_id, broker_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let result = db.brokers.delete(app_id, claim.user_id, broker_id);

    match result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}
