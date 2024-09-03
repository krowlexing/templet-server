use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_utils::{unwrap_json, Claim};
use serde::{Deserialize, Serialize};

use crate::db::{
    apps::{AppStatus, NewApp},
    Db,
};

use super::tokens::AppClaim;

pub async fn all_apps(State(db): State<Db>) -> impl IntoResponse {
    match db.apps.select_all() {
        Ok(apps) => (StatusCode::OK, serde_json::to_string(&apps).unwrap()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    }
}

/**
    some user requested to create new app
*/
#[derive(Serialize, Deserialize)]
pub struct NewAppRequest {
    pub title: String,
    pub description: String,
    pub weblink: String,
    pub version: String,
    pub public: bool,
    pub status: AppStatus,
}

impl NewAppRequest {
    pub fn with_author(self, author_id: i32) -> NewApp {
        let Self {
            title,
            description,
            weblink,
            version,
            public,
            status,
        } = self;

        NewApp {
            author_id,
            title,
            description,
            weblink,
            version,
            public,
            status,
        }
    }
}

pub async fn new_app(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Json(new_app): Json<NewAppRequest>,
) -> impl IntoResponse {
    match db.apps.insert(new_app.with_author(claim.user_id)) {
        Ok(apps) => (StatusCode::OK, serde_json::to_string(&apps).unwrap()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    }
}

#[derive(Deserialize)]
pub struct AppSearchQuery {
    q: String,
}

pub async fn search(
    State(db): State<Db>,
    Query(query): Query<AppSearchQuery>,
) -> impl IntoResponse {
    let title = query.q;
    match db.apps.search_by_name(title.clone()) {
        Ok(apps) => (StatusCode::OK, unwrap_json(&apps)),
        Err(sql) => {
            println!("sql error happend while searhing apps by title '{title}'\n {sql:?}");

            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        }
    }
}

pub async fn by_id(
    State(db): State<Db>,
    Claim(claim): AppClaim,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match db.apps.by_id_for_user(id, claim.user_id) {
        Ok(Some(app)) => (StatusCode::OK, unwrap_json(&app)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND).into_response(),
        Err(sql) => {
            println!("sql error happend while searching app with id '{id}'\n {sql:?}");

            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}
