use async_trait::async_trait;
use tracing::info;

#[async_trait]
pub trait AlertProvider: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> anyhow::Result<()>;
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()>;
}

pub struct MockAlertProvider;

#[async_trait]
impl AlertProvider for MockAlertProvider {
    async fn send_sms(&self, to: &str, message: &str) -> anyhow::Result<()> {
        info!("--- [MOCK SMS ALERT] ---");
        info!("To: {}", to);
        info!("Message: {}", message);
        info!("-----------------------");
        Ok(())
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
        info!("--- [MOCK EMAIL ALERT] ---");
        info!("To: {}", to);
        info!("Subject: {}", subject);
        info!("Body: {}", body);
        info!("-------------------------");
        Ok(())
    }
}
