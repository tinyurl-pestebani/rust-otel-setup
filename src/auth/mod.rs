mod unauthenticated;
mod gcp;
pub mod layer;

use std::fmt::Debug;
use async_trait::async_trait;
use anyhow::Result;


/// Trait for obtaining authentication tokens.
#[async_trait]
pub trait GetToken: Debug + Send + Sync {
    /// Asynchronously retrieves authentication headers.
    async fn get_auth_headers(&self) -> Result<Vec<(String, String)>>;
}
