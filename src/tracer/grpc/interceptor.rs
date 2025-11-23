use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tonic::metadata::{Ascii, MetadataKey};
use crate::auth::GetToken;


/// A gRPC interceptor that adds authorization metadata to requests.
#[derive(Clone)]
pub struct TonicInterceptor {
    token_provider: Arc<dyn GetToken>,
}


/// Implementation of TonicInterceptor
impl TonicInterceptor {
    /// Creates a new instance of `TonicInterceptor`.
    /// # Arguments
    /// * `token_provider` - An `Arc<dyn GetToken>` to provide access tokens.
    /// # Returns
    /// A new `TonicInterceptor` instance.
    pub fn new(token_provider: Arc<dyn GetToken>) -> Self {
        Self { token_provider }
    }
}


/// Implementation of the gRPC interceptor trait for TonicInterceptor
impl tonic::service::Interceptor for TonicInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> anyhow::Result<tonic::Request<()>, tonic::Status> {
        let rt = Runtime::new()?;
        let headers = rt.block_on(async {self.token_provider.get_auth_headers().await}).map_err(|err| {tonic::Status::unauthenticated(format!("{}", err))})?;

        for (key, value) in headers {
            let k: MetadataKey<Ascii> = MetadataKey::from_str(key.as_str()).map_err(|err| tonic::Status::unauthenticated(format!("{}", err)))?;
            req.metadata_mut().insert(k, value.parse().map_err(|e| tonic::Status::internal(format!("Failed to parse metadata value: {}", e)))?);
        }

        match std::env::var("GOOGLE_PROJECT_ID") {
            Ok(val) => {
                req.metadata_mut().insert("x-goog-user-project", val.parse().map_err(|e| tonic::Status::internal(format!("Failed to parse metadata value: {}", e)))?);
            },
            Err(_) => {},
        };

        Ok(req)
    }
}
