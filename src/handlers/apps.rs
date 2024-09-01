use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_utils::unwrap_json;
use serde::{Deserialize, Serialize};

use crate::db::{
    apps::{AppStatus, NewApp},
    Db,
};

use super::tokens::Claim;

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
    pub fn with_author(self, author: String) -> NewApp {
        let Self {
            title,
            description,
            weblink,
            version,
            public,
            status,
        } = self;

        NewApp {
            author,
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
    Claim(claim): Claim,
    Json(new_app): Json<NewAppRequest>,
) -> impl IntoResponse {
    match db.apps.insert(new_app.with_author(claim.username)) {
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
    Claim(claim): Claim,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    match db.apps.by_id(id) {
        Ok(Some(app)) => {
            if app.public || app.author == claim.username {
                (StatusCode::OK, unwrap_json(&app))
            } else {
                (StatusCode::NOT_FOUND, String::new())
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, String::new()),
        Err(sql) => {
            println!("sql error happend while searching app with id '{id}'\n {sql:?}");

            (StatusCode::INTERNAL_SERVER_ERROR, String::new())
        }
    }
}
