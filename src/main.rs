use ::core::future::Future;
use ::core::marker::Send;
use ::core::pin::Pin;
use std::sync::Arc;

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{async_trait, Json};
use axum::{routing::post, Router};
use db::users::{NewUser, User};
use db::{Db, SqliteDb};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
pub mod db;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = Arc::new(SqliteDb::new("sqlite.db".to_string()).unwrap());

    db.init().unwrap();

    let app = Router::new()
        .route("/", get(handle_index))
        .route("/convert", post(process_markdown))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/test", get(test_auth))
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
    [$name:ident($db:ident $($others:ident: $other_ty:ty),*, $body:ident : $body_type:ty) $body_block:block] => {
        async fn $name(State($db): State<Db>, $($others : $other_ty),* Json($body): Json<$body_type>) -> impl IntoResponse $body_block
    };
}

use axum::http::StatusCode;
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
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

async fn test_auth(Claim(claim): Claim) -> impl IntoResponse {
    format!("hey there {}", claim.username)
}

pub fn generate_claim(username: String) -> String {
    let claim = UserClaim { username };

    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).unwrap();
    claim.sign_with_key(&key).unwrap()
}

pub fn check_claim(token: &str) -> Result<UserClaim, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).unwrap();
    token.verify_with_key(&key)
}

pub struct Claim(UserClaim);

#[async_trait]
impl<S> FromRequestParts<S> for Claim
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get("Authorization")
            .ok_or(StatusCode::UNAUTHORIZED)
            .and_then(|auth_content| auth_content.to_str().map_err(|_| StatusCode::UNAUTHORIZED))
            .and_then(|str| check_claim(str).map_err(|_| StatusCode::UNAUTHORIZED))
            .map(Claim)
    }
}
