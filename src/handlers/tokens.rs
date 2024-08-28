use axum::async_trait;
use axum::http::request::Parts;

use axum::http::StatusCode;

use axum::extract::FromRequestParts;

use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use hmac::{Hmac, Mac};

pub(crate) const KEY: &[u8] = b"super-secret";

pub fn generate_claim(username: String) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).unwrap();
    UserClaim { username }.sign_with_key(&key).unwrap()
}

pub fn check_claim(token: &str) -> Result<UserClaim, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).unwrap();
    token.verify_with_key(&key)
}

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
    pub(crate) username: String,
}

pub struct Claim(pub UserClaim);

#[async_trait]
impl<S> FromRequestParts<S> for Claim
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get("Authorization")
            .ok_or(StatusCode::UNAUTHORIZED)
            .and_then(|auth_content| auth_content.to_str().map_err(|_| StatusCode::UNAUTHORIZED))
            .and_then(|str| check_claim(str).map_err(|_| StatusCode::UNAUTHORIZED))
            .map(Claim)
    }
}
