use super::errors::FitbitError;
use super::types::*;
use reqwest::Client;
use std::time::Duration;

const FITBIT_API_BASE: &str = "https://api.fitbit.com";
const REQUEST_TIMEOUT_SECS: u64 = 10;

#[derive(Clone)]
pub struct FitbitWebAPIClient {
    client: Client,
    client_id: String,
    client_secret: String,
}

impl FitbitWebAPIClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_default();

        Self {
            client,
            client_id,
            client_secret,
        }
    }

    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("FITBIT_CLIENT_ID").ok()?;
        let client_secret = std::env::var("FITBIT_CLIENT_SECRET").ok()?;
        Some(Self::new(client_id, client_secret))
    }

    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<FitbitAuth, FitbitError> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", &self.client_id),
        ];

        let response = self
            .client
            .post(format!("{}/oauth2/token", FITBIT_API_BASE))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FitbitError::AuthenticationFailed(format!(
                "Token exchange failed with status: {}",
                response.status()
            )));
        }

        let token_response: FitbitTokenResponse = response
            .json()
            .await
            .map_err(|e| FitbitError::InvalidResponse(e.to_string()))?;

        let scopes: Vec<String> = token_response
            .scope
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(FitbitAuth {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            user_id: token_response.user_id,
            expires_at: chrono::Utc::now().timestamp() + token_response.expires_in,
            scope: scopes,
        })
    }

    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<FitbitAuth, FitbitError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .client
            .post(format!("{}/oauth2/token", FITBIT_API_BASE))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FitbitError::TokenExpired(
                "Refresh token is invalid or expired".to_string(),
            ));
        }

        let token_response: FitbitTokenResponse = response
            .json()
            .await
            .map_err(|e| FitbitError::InvalidResponse(e.to_string()))?;

        let scopes: Vec<String> = token_response
            .scope
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(FitbitAuth {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            user_id: token_response.user_id,
            expires_at: chrono::Utc::now().timestamp() + token_response.expires_in,
            scope: scopes,
        })
    }

    pub async fn get_heart_rate_data(
        &self,
        access_token: &str,
        user_id: &str,
        date: &str,
    ) -> Result<FitbitHeartRateData, FitbitError> {
        let url = format!(
            "{}/1/user/{}/activities/heart/date/{}/1d.json",
            FITBIT_API_BASE, user_id, date
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        self.handle_api_response(response).await
    }

    pub async fn get_sleep_data(
        &self,
        access_token: &str,
        user_id: &str,
        date: &str,
    ) -> Result<Vec<SleepSession>, FitbitError> {
        let url = format!(
            "{}/1.2/user/{}/sleep/date/{}.json",
            FITBIT_API_BASE, user_id, date
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        self.handle_api_response(response).await
    }

    pub async fn get_activity_data(
        &self,
        access_token: &str,
        user_id: &str,
        date: &str,
    ) -> Result<DailyActivity, FitbitError> {
        let url = format!(
            "{}/1/user/{}/activities/date/{}.json",
            FITBIT_API_BASE, user_id, date
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        self.handle_api_response(response).await
    }

    pub async fn get_hrv_data(
        &self,
        access_token: &str,
        user_id: &str,
        date: &str,
    ) -> Result<Vec<HRVReading>, FitbitError> {
        let url = format!(
            "{}/1/user/{}/hrv/date/{}.json",
            FITBIT_API_BASE, user_id, date
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        self.handle_api_response(response).await
    }

    async fn handle_api_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, FitbitError> {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(FitbitError::AuthenticationFailed(
                "Access token is invalid or expired".to_string(),
            ));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(FitbitError::RateLimitExceeded);
        }

        if !status.is_success() {
            return Err(FitbitError::ApiRequestFailed(format!(
                "API returned status: {}",
                status
            )));
        }

        response
            .json()
            .await
            .map_err(|e| FitbitError::InvalidResponse(e.to_string()))
    }
}
