#[derive(Clone, Debug)]
pub struct UnauthenticatedInterceptor;


/// Create a new instance of UnauthenticatedInterceptor.
impl UnauthenticatedInterceptor {
    pub fn new() -> Self {
        Self {}
    }
}


/// Implement the Interceptor trait for UnauthenticatedInterceptor. The request remains unchanged.
impl tonic::service::Interceptor for UnauthenticatedInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        Ok(req)
    }
}
