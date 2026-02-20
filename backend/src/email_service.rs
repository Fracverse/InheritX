use crate::api_error::ApiError;
use tracing::info;

/// Email service for sending OTPs and notifications
/// TODO: Integrate with actual email provider (SendGrid, AWS SES, Mailgun, etc.)
pub struct EmailService;

impl EmailService {
    /// Send OTP email to user
    pub async fn send_otp_email(to: &str, otp: &str) -> Result<(), ApiError> {
        let subject = "Your InheritX Verification Code";
        let body = format!(
            "Your InheritX verification code is: {}\n\n\
             This code will expire in 5 minutes.\n\n\
             If you didn't request this code, please ignore this email.",
            otp
        );

        Self::send_email(to, subject, &body).await
    }

    /// Generic email sending function
    /// TODO: Replace with actual email service implementation
    async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), ApiError> {
        // For development: just log the email
        info!("📧 Email would be sent:");
        info!("   To: {}", to);
        info!("   Subject: {}", subject);
        info!("   Body: {}", body);

        // TODO: Implement actual email sending
        // Example with SendGrid:
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("https://api.sendgrid.com/v3/mail/send")
        //     .header("Authorization", format!("Bearer {}", api_key))
        //     .json(&email_payload)
        //     .send()
        //     .await?;

        // Example with AWS SES:
        // let ses_client = SesClient::new(Region::UsEast1);
        // ses_client.send_email(send_email_request).await?;

        Ok(())
    }
}
