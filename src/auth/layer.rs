use std::sync::Arc;
use crate::auth::GetToken;
use crate::config::AuthConfig;
use crate::auth::unauthenticated::Unauthenticated;
use crate::auth::gcp::GcpAuthProvider;


/// Creates a new token provider based on the given authentication configuration.
pub fn new_gen_token(config: &AuthConfig) -> Arc<dyn GetToken> {
    match config {
        AuthConfig::Unauthenticated => Arc::new(Unauthenticated::new()),
        AuthConfig::GCPAuth(conf) => Arc::new(GcpAuthProvider::new_with_default(conf)),
    }
}