use crate::emergency_access::EmergencyAccessService;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Background job service for emergency access monitoring
pub struct EmergencyAccessJobService;

impl EmergencyAccessJobService {
    /// Start the background job that periodically checks for expiring access
    /// and marks expired access as expired
    pub fn start(db: Arc<PgPool>) {
        tokio::spawn(async move {
            // Run every hour
            let mut interval = tokio::time::interval(Duration::from_secs(3600));

            loop {
                interval.tick().await;

                // Check for expiring access (within 24 hours)
                match EmergencyAccessService::check_expiring_access(&db).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Emergency access expiration check: {} notifications sent", count);
                        }
                    }
                    Err(e) => {
                        error!("Error checking expiring emergency access: {}", e);
                    }
                }

                // Mark access that has already expired
                match EmergencyAccessService::mark_expired_access(&db).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Marked {} emergency access records as expired", count);
                        }
                    }
                    Err(e) => {
                        error!("Error marking expired emergency access: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergency_access_job_service_exists() {
        // Verify the service can be instantiated
        let _service = EmergencyAccessJobService;
    }
}
