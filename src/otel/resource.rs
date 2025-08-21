use std::sync::OnceLock;
use opentelemetry_sdk::Resource;


/// Returns a singleton `Resource` instance.
///
/// The resource is initialized with the service name.
///
/// # Arguments
///
/// * `service_name` - The name of the service.
pub fn get_resource(service_name: &String) -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name(service_name.clone())
                .build()
        })
        .clone()
}