use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::{routing::post, Router};
use db::users::NewUser;
use db::{Db, SqliteDb};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
pub mod db;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = Arc::new(SqliteDb::new("sqlite.db".to_string()).unwrap());

    let app = Router::new()
        .route("/", get(handle_index))
        .route("/convert", post(process_markdown))
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(db);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_index() -> &'static str {
    "Hello World!"
}

async fn process_markdown(body: String) -> impl IntoResponse {
    "hello"
}

#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    name: String,
    username: String,
    password: String,
}
macro_rules! handle_request {
    [$name:ident($db:ident, $body:ident : $body_type:ty) $body_block:block] => {
        async fn $name(State($db): State<Db>, Json($body): Json<$body_type>) -> impl IntoResponse $body_block
    };
}

use axum::http::StatusCode;
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
struct UserClaim {
    username: String,
}

const ERROR: &str = "{ \"status\": \"error\" }";
const KEY: &[u8] = b"super-secret";

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

handle_request![login(db, req: LoginRequest) {
    use db::users::LoginError::*;

    let user = db.users.find_user(&req.username, &req.password);

    match user {
        Ok(_user) => {
            (StatusCode::OK, generate_claim(req.username))
        },
        Err(UserNotFound) => {
            println!("registering user, but username already exists");
            (StatusCode::NOT_FOUND, ERROR.to_string())
        },
        Err(WrongPassword) => {
            (StatusCode::FORBIDDEN, ERROR.to_string())
        }
        Err(SqliteError(e)) => {
            println!("500 ERROR while executing 'find_user': Sqlite reported error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, ERROR.to_string())
        }
    }
}];

handle_request![register(db, req: RegisterRequest) {

    let user = db.users.find_user_by_name(&req.username);
    if user.is_ok() {
        return (StatusCode::NOT_FOUND, "sorry".to_string());
    }


    let result = NewUser::new(req.name, req.username.clone(), req.password)
        .map(|user| db.users.insert(user));

    match result {
        Ok(_) => (StatusCode::OK, generate_claim(req.username)),
        Err(_) => (StatusCode::NOT_FOUND, "".to_string())
    }
}];

pub fn generate_claim(username: String) -> String {
    let claim = UserClaim { username };

    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).unwrap();
    claim.sign_with_key(&key).unwrap()
}
