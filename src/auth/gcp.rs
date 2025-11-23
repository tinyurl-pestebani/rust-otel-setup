use std::sync::Arc;
use async_trait::async_trait;
use google_cloud_auth::credentials::{Builder, CacheableResource};
use tokio::sync::RwLock;
use tonic::codegen::http::header::AUTHORIZATION;
use tonic::codegen::http::HeaderMap;
use anyhow::Result;
use crate::auth::GetToken;
use crate::config::GCPAuthConfig;

#[derive(Debug, Clone)]
pub struct GcpAuthProvider {
    token: Arc<RwLock<String>>,
    last_refresh: Arc<RwLock<std::time::SystemTime>>,
    project_id: String,
}


/// Extracts the token from the provided headers.
/// # Arguments
/// /// * `headers` - A `CacheableResource` containing the headers from which to extract the
/// token.
/// # Returns
/// An `Option<String>` containing the extracted token if present.
fn get_token_from_headers(headers: CacheableResource<HeaderMap>) -> Option<String> {
    match headers {
        CacheableResource::New { data, .. } => data
            .get(AUTHORIZATION)
            .and_then(|token_value| token_value.to_str().ok())
            .and_then(|s| s.split_whitespace().nth(1))
            .map(|s| s.to_string()),
        CacheableResource::NotModified => None,
    }
}


impl GcpAuthProvider {
    /// Creates a new instance of `GcpAuthProvider`.
    /// # Arguments
    /// * `token` - An `Arc<RwLock<String>>` to hold the access token.
    /// * `last_refresh` - An `Arc<RwLock<SystemTime>>` to track the last refresh time.
    /// * `project_id` - A `String` representing the GCP project ID.
    /// # Returns
    /// A new `GcpAuthProvider` instance.
    fn new(token: Arc<RwLock<String>>, last_refresh: Arc<RwLock<std::time::SystemTime>>, project_id: String) -> Self {
        Self { token , last_refresh, project_id}
    }

    /// Creates a new instance of `GcpAuthProvider` with default values.
    /// The token is initialized as an empty string, and the last refresh time is set to
    /// the UNIX epoch.
    /// # Arguments
    /// * `config` - A reference to `GCPAuthConfig` containing configuration details
    /// # Returns
    /// A new `GcpAuthProvider` instance with default values.
    pub fn new_with_default(config: &GCPAuthConfig) -> Self {
        let token: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
        let last_refresh: Arc<RwLock<std::time::SystemTime>> = Arc::new(RwLock::new(std::time::SystemTime::from(std::time::UNIX_EPOCH)));
        Self::new(token, last_refresh, config.project_id.clone())

    }

    /// Retrieves a new access token using GCP credentials.
    /// # Returns
    /// A `Result<String>` containing the new access token or an error if retrieval fails.
    async fn get_new_token() -> Result<String> {
        // Build the credentials using the default builder
        let credentials = Builder::default().build();

        // Get the headers containing the access token
        let headers = credentials
            .map_err(|e| anyhow::anyhow!("Error creating auth credentials: {:?}", e))?
            .headers(tonic::Extensions::new())
            .await
            .map_err(|e| anyhow::anyhow!("Error creating auth headers: {:?}", e))?;


        let token = get_token_from_headers(headers);

        match token {
            Some(t) => Ok(t),
            None => Err(anyhow::anyhow!("Failed to get token from headers")),
        }
    }

    /// Authenticates and updates the access token.
    /// # Returns
    /// A `Result<()>` indicating success or failure of the authentication process.
    async fn authenticate(&self) -> anyhow::Result<()> {
        let token = Self::get_new_token().await.map_err(|e| anyhow::anyhow!("Error retrieving new token: {:?}", e))?;

        let mut w = self.token.write().await;
        *w = token;
        let mut lr = self.last_refresh.write().await;
        *lr = std::time::SystemTime::now();
        Ok(())
    }

    /// Retrieves the current access token and updates it if necessary.
    /// If more than 10 minutes have passed since the last refresh, the token is refreshed.
    /// # Returns
    /// A `Result<String>` containing the current access token or an error if retrieval fails.
    async fn get_and_update_token(&self) -> Result<String> {
        let last_refresh = self.last_refresh.read().await;
        let elapsed = last_refresh.elapsed().unwrap_or(std::time::Duration::new(601,0));
        drop(last_refresh);

        // If more than 10 minutes have passed since last refresh, refresh the token
        if elapsed.as_secs() > 600 {
            self.authenticate().await.map_err(|e| anyhow::anyhow!("Error authenticating token: {:?}", e))?;
        }

        Ok(self.token.read().await.clone())
    }
}


/// Implements the `GetToken` trait for `GcpAuthProvider`.
#[async_trait]
impl GetToken for GcpAuthProvider {
    async fn get_auth_headers(&self) -> Result<Vec<(String, String)>> {
        let token = self.get_and_update_token().await?;
        Ok(vec![("authorization".to_string(), format!("Bearer {}", token)),
                ("x-goog-user-project".to_string(), self.project_id.clone())])
    }
}
