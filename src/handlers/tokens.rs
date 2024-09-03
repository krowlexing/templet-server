use axum_utils::{jwt_sign, jwt_verify, Claim, VerifiebleClaim};

use serde::{Deserialize, Serialize};

pub(crate) const KEY: &[u8] = b"super-secret";

pub type AppClaim = Claim<UserClaim>;

#[derive(Serialize, Deserialize)]
pub struct UserClaim {
    pub(crate) user_id: i32,
}

impl VerifiebleClaim for UserClaim {
    fn check(claim: &str) -> Result<Self, jwt::Error>
    where
        Self: Sized,
    {
        jwt_verify(claim, KEY)
    }

    fn sign(self) -> String {
        jwt_sign(self, KEY).unwrap()
    }
}
