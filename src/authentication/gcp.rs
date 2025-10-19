use std::sync::Arc;
use google_cloud_auth::credentials::{Builder, CacheableResource};
use tokio::sync::RwLock;
use tokio::runtime::Runtime;
use anyhow::Result;
use tonic::codegen::http::header::AUTHORIZATION;
use tonic::codegen::http::HeaderMap;


/// `GCPAuthenticationInterceptor` is a gRPC interceptor that handles authentication
/// using Google Cloud Platform (GCP) credentials.
/// It automatically retrieves and refreshes access tokens as needed.
#[derive(Clone, Debug)]
pub struct GCPAuthenticationInterceptor {
    token: Arc<RwLock<String>>,
    last_refresh: Arc<RwLock<std::time::SystemTime>>,
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


/// Implementation of GCPAuthenticationInterceptor
/// Provides methods for creating a new interceptor, authenticating,
/// and retrieving/updating tokens.
impl GCPAuthenticationInterceptor {
    /// Creates a new instance of `GCPAuthenticationInterceptor`.
    /// # Arguments
    /// * `token` - An `Arc<RwLock<String>>` to hold the access token.
    /// * `last_refresh` - An `Arc<RwLock<SystemTime>>` to track the last refresh time.
    /// # Returns
    /// A new `GCPAuthenticationInterceptor` instance.
    fn new(token: Arc<RwLock<String>>, last_refresh: Arc<RwLock<std::time::SystemTime>>) -> Self {
        Self { token , last_refresh}
    }

    /// Creates a new instance of `GCPAuthenticationInterceptor` with default values.
    /// The token is initialized as an empty string, and the last refresh time is set to
    /// the UNIX epoch.
    /// # Returns
    /// A new `GCPAuthenticationInterceptor` instance with default values.
    pub fn new_with_default() -> Self {
        let token: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
        let last_refresh: Arc<RwLock<std::time::SystemTime>> = Arc::new(RwLock::new(std::time::SystemTime::from(std::time::UNIX_EPOCH)));
        Self::new(token, last_refresh)

    }

    /// Retrieves a new access token using GCP credentials.
    /// # Returns
    /// A `Result<String>` containing the new access token or an error if retrieval fails.
    async fn get_new_token() -> Result<String> {
        // Build the credentials using the default builder
        let credentials = Builder::default().build();

        // Get the headers containing the access token
        let headers = credentials?
            .headers(tonic::Extensions::new())
            .await?;


        let token = get_token_from_headers(headers);

        match token {
            Some(t) => Ok(t),
            None => Err(anyhow::anyhow!("Failed to get token from headers")),
        }
    }

    /// Authenticates and updates the access token.
    /// # Returns
    /// A `Result<()>` indicating success or failure of the authentication process.
    async fn authenticate(&self) -> Result<()>{
        let token = Self::get_new_token().await?;

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
            self.authenticate().await?;
        }

        Ok(self.token.read().await.clone())
    }
}


/// Implementation of the tonic Interceptor trait for GCPAuthenticationInterceptor
impl tonic::service::Interceptor for GCPAuthenticationInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let rt = Runtime::new()?;
        let token = rt.block_on(async {self.get_and_update_token().await}).map_err(|err| {tonic::Status::unauthenticated(format!("{}", err))})?;

        let metadata_value = format!("Bearer {}", token);

        req.metadata_mut().insert(
            "authorization",
            metadata_value.parse().map_err(|e| {
                tonic::Status::internal(format!("Failed to parse metadata value: {}", e))
            })?,
        );

        let google_project_id = std::env::var("GOOGLE_PROJECT_ID").map_err(|e| {
            tonic::Status::internal(format!("Failed to get GOOGLE_PROJECT_ID: {}", e))
        })?;

        req.metadata_mut().insert(
            "x-goog-user-project",
            google_project_id.parse().unwrap());
        Ok(req)
    }
}
