use crate::api_error::ApiError;
use crate::config::EmailConfig;

/// Email service for sending OTPs and notifications
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Send OTP via email
    pub async fn send_otp(&self, to_email: &str, otp: &str) -> Result<(), ApiError> {
        // For now, we'll log the OTP (in production, use a real email service)
        tracing::info!(
            "Sending OTP to {}: {} (expires in 5 minutes)",
            to_email,
            otp
        );

        // TODO: Integrate with actual email service (SendGrid, AWS SES, etc.)
        // Example with reqwest:
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(&self.config.smtp_host)
        //     .json(&json!({
        //         "to": to_email,
        //         "subject": "Your InheritX 2FA Code",
        //         "body": format!("Your verification code is: {}", otp)
        //     }))
        //     .send()
        //     .await?;

        Ok(())
    }

    /// Send notification email
    pub async fn send_notification(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), ApiError> {
        tracing::info!("Sending notification to {}: {}", to_email, subject);

        // TODO: Integrate with actual email service
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_otp() {
        let config = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "test@example.com".to_string(),
            smtp_password: "password".to_string(),
            from_email: "noreply@inheritx.com".to_string(),
        };

        let service = EmailService::new(config);
        let result = service.send_otp("user@example.com", "123456").await;
        assert!(result.is_ok());
    }
}
