use tonic::service::Interceptor;
use crate::config::OTLPTraceInterceptor;

pub(crate) mod unauthenticated;
pub(crate) mod gcp;


/// A common interceptor that can represent different authentication strategies.
#[derive(Clone, Debug)]
pub enum CommonInterceptor {
    Unauthenticated(unauthenticated::UnauthenticatedInterceptor),
    GCP(gcp::GCPAuthenticationInterceptor),
}


/// Implement the Interceptor trait for CommonInterceptor.
/// This allows it to delegate the call to the appropriate underlying interceptor.
impl Interceptor for CommonInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        match self {
            CommonInterceptor::Unauthenticated(interceptor) => interceptor.call(req),
            CommonInterceptor::GCP(interceptor) => interceptor.call(req),
        }
    }
}


/// Factory method to create a CommonInterceptor based on the provided configuration.
impl CommonInterceptor {
    /// Create a new CommonInterceptor based on the OTLPTraceInterceptor configuration.
    /// /// # Arguments
    /// /// * `interceptor_config` - The configuration specifying which interceptor to use.
    /// /// # Returns
    /// /// A CommonInterceptor instance.
    pub fn new(interceptor_config: &OTLPTraceInterceptor) -> CommonInterceptor {
        match interceptor_config {
            OTLPTraceInterceptor::None => CommonInterceptor::Unauthenticated(unauthenticated::UnauthenticatedInterceptor::new()),
            OTLPTraceInterceptor::GCP => CommonInterceptor::GCP(gcp::GCPAuthenticationInterceptor::new_with_default()),
        }
    }
}
