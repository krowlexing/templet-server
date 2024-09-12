use std::sync::Arc;

use axum::routing::{delete, get};
use axum::{routing::post, Router};

use db::SqliteDb;

use handlers::apps::{self, all_apps, new_app};
use handlers::auth::{login, register};
use handlers::{app_users, operators, users};

pub mod db;
pub mod handlers;

#[tokio::main]
async fn main() {
    let db = Arc::new(SqliteDb::new("sqlite.db".to_string()).unwrap());

    db.init().unwrap();

    let app_users_router = Router::new()
        .route("/", get(app_users::all))
        .route("/", post(app_users::create))
        .route("/:user_id", delete(app_users::delete));

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/users/search", get(users::search))
        .route("/apps/:app_id/operators", get(operators::all))
        .route("/apps/:app_id/operators", post(operators::create))
        .route(
            "/apps/:app_id/operators/:operator_id",
            delete(operators::delete),
        )
        .nest("/apps/:app_id/users", app_users_router)
        .route("/apps/:app_id/info", get(apps::by_id))
        .route("/apps/search", get(apps::search))
        .route("/apps/", get(all_apps))
        .route("/apps/", post(new_app))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
