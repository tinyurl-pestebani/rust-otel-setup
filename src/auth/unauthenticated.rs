use async_trait::async_trait;
use anyhow::Result;
use crate::auth::GetToken;


/// An authentication provider that does not provide any token.
#[derive(Debug, Clone)]
pub struct Unauthenticated;


/// Creates a new instance of `Unauthenticated`.
impl Unauthenticated {
    pub fn new() -> Self {
        Self {}
    }
}


/// Implements the `GetToken` trait for `Unauthenticated`.
#[async_trait]
impl GetToken for Unauthenticated {
    async fn get_auth_headers(&self) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
}
