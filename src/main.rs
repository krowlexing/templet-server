use std::sync::Arc;

use axum::routing::get;
use axum::{routing::post, Router};

use db::SqliteDb;

use handlers::apps::{all_apps, new_app};
use handlers::auth::{login, register};

pub mod db;
pub mod handlers;

#[tokio::main]
async fn main() {
    let db = Arc::new(SqliteDb::new("sqlite.db".to_string()).unwrap());

    db.init().unwrap();

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/apps/", get(all_apps))
        .route("/apps/", post(new_app))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
