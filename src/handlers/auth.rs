use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, users::NewUser, Db},
    handlers::tokens::generate_claim,
};

macro_rules! handle_request {
    [$name:ident($db:ident $($others:ident: $other_ty:ty),*, $body:ident : $body_type:ty) $body_block:block] => {
        pub async fn $name(State($db): State<Db>, $($others : $other_ty),* Json($body): Json<$body_type>) -> impl IntoResponse $body_block
    };
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    name: String,
    username: String,
    password: String,
}

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

pub const ERROR: &str = "{ \"status\": \"error\" }";
