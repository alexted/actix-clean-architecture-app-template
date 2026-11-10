//! Data transfer objects of the service context storage.

use crate::infrastructure::clients::postgres::models::service_context::ServiceContextDiesel;

/// The service context as stored in the database: the runtime state of this service.
#[derive(Debug, Clone)]
pub struct ServiceContextDTO {
    pub id: i32,
    pub maintenance: bool,
}

/// Convert an ORM row into its DTO.
impl From<ServiceContextDiesel> for ServiceContextDTO {
    fn from(service_context: ServiceContextDiesel) -> Self {
        ServiceContextDTO {
            id: service_context.id,
            maintenance: service_context.maintenance,
        }
    }
}

/// Convert a DTO into its ORM row.
impl From<ServiceContextDTO> for ServiceContextDiesel {
    fn from(service_context: ServiceContextDTO) -> Self {
        ServiceContextDiesel {
            id: service_context.id,
            maintenance: service_context.maintenance,
        }
    }
}
