use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_utils::VerifiebleClaim;
use serde::{Deserialize, Serialize};

use crate::db::{self, users::NewUser, Db};

use super::tokens::UserClaim;

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
    let username = req.username;

    let user = db.users.find_user_by_name(&username);
    if user.is_ok() {
        return (StatusCode::NOT_FOUND, "sorry".to_string());
    }


    let result = NewUser::new(req.name, username.clone(), req.password)
        .map(|user| db.users.insert(user));

    match result {
        // Insert successful
        Ok(Ok(user_id)) => (StatusCode::OK, UserClaim { user_id }.sign()),
        // Insert failed
        Ok(Err(_)) => {
            println!("Error while handling '/register', insert failed");
            (StatusCode::NOT_FOUND, "".to_string()) },
        // Validation failed
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
    let username = req.username;
    let user = db.users.find_user(&username, &req.password);

    match user {
        Ok(user) => {
            (StatusCode::OK, UserClaim { user_id: user.id }.sign())
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
