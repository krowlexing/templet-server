use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_utils::unwrap_json;
use serde::Deserialize;

use crate::{db::Db, handlers::auth::ERROR};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search(State(db): State<Db>, Query(query): Query<SearchQuery>) -> impl IntoResponse {
    match db.users.search(&query.q) {
        Ok(users) => (StatusCode::OK, unwrap_json(&users)),
        Err(e) => {
            println!("500 ERROR while executing 'users::search': Sqlite reported error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, ERROR.to_string())
        }
    }
}
