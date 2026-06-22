use super::errors::DatabaseError;
use super::types::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelativeMatch {
    pub external_id: String,
    pub relationship: String,
    pub shared_centimorgans: f64,
    pub match_confidence: f64,
    pub common_ancestors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedGeneticPayload {
    pub profile_id: String,
    pub snp_data: Vec<SNPVariant>,
    pub ancestry_composition: AncestryBreakdown,
    pub privacy_level: PrivacyLevel,
}

#[async_trait]
pub trait ExternalGeneticService: Send + Sync {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthToken, DatabaseError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, DatabaseError>;
    async fn fetch_relative_matches(&self, access_token: &str) -> Result<Vec<RelativeMatch>, DatabaseError>;
    async fn upload_genetic_payload(&self, access_token: &str, payload: &SharedGeneticPayload) -> Result<(), DatabaseError>;
}

#[derive(Clone)]
pub struct TwentyThreeAndMeClient {
    client: Client,
    client_id: String,
    client_secret: String,
    auth_base: String,
}

impl TwentyThreeAndMeClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            client_id,
            client_secret,
            auth_base: "https://api.23andme.com".to_string(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("TWENTYTHREE_AND_ME_CLIENT_ID").ok()?;
        let client_secret = std::env::var("TWENTYTHREE_AND_ME_CLIENT_SECRET").ok()?;
        Some(Self::new(client_id, client_secret))
    }
}

#[async_trait]
impl ExternalGeneticService for TwentyThreeAndMeClient {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "23andMe token exchange failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "23andMe refresh failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn fetch_relative_matches(&self, access_token: &str) -> Result<Vec<RelativeMatch>, DatabaseError> {
        let response = self
            .client
            .get(format!("{}/relatives", self.auth_base))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "23andMe relative fetch failed: {}",
                response.status()
            )));
        }

        let matches: Vec<RelativeMatch> = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(matches)
    }

    async fn upload_genetic_payload(&self, access_token: &str, payload: &SharedGeneticPayload) -> Result<(), DatabaseError> {
        let response = self
            .client
            .post(format!("{}/genetic_sharing", self.auth_base))
            .bearer_auth(access_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "23andMe upload failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct AncestryDNAClient {
    client: Client,
    client_id: String,
    client_secret: String,
    auth_base: String,
}

impl AncestryDNAClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            client_id,
            client_secret,
            auth_base: "https://api.ancestrydna.com".to_string(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("ANCESTRY_DNA_CLIENT_ID").ok()?;
        let client_secret = std::env::var("ANCESTRY_DNA_CLIENT_SECRET").ok()?;
        Some(Self::new(client_id, client_secret))
    }
}

#[async_trait]
impl ExternalGeneticService for AncestryDNAClient {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "AncestryDNA token exchange failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "AncestryDNA refresh failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn fetch_relative_matches(&self, access_token: &str) -> Result<Vec<RelativeMatch>, DatabaseError> {
        let response = self
            .client
            .get(format!("{}/relatives", self.auth_base))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "AncestryDNA relative fetch failed: {}",
                response.status()
            )));
        }

        let matches: Vec<RelativeMatch> = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(matches)
    }

    async fn upload_genetic_payload(&self, access_token: &str, payload: &SharedGeneticPayload) -> Result<(), DatabaseError> {
        let response = self
            .client
            .post(format!("{}/genetic_sharing", self.auth_base))
            .bearer_auth(access_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "AncestryDNA upload failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct MyHeritageClient {
    client: Client,
    client_id: String,
    client_secret: String,
    auth_base: String,
}

impl MyHeritageClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            client_id,
            client_secret,
            auth_base: "https://api.myheritage.com".to_string(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("MYHERITAGE_CLIENT_ID").ok()?;
        let client_secret = std::env::var("MYHERITAGE_CLIENT_SECRET").ok()?;
        Some(Self::new(client_id, client_secret))
    }
}

#[async_trait]
impl ExternalGeneticService for MyHeritageClient {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "MyHeritage token exchange failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "MyHeritage refresh failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn fetch_relative_matches(&self, access_token: &str) -> Result<Vec<RelativeMatch>, DatabaseError> {
        let response = self
            .client
            .get(format!("{}/relatives", self.auth_base))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "MyHeritage relative fetch failed: {}",
                response.status()
            )));
        }

        let matches: Vec<RelativeMatch> = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(matches)
    }

    async fn upload_genetic_payload(&self, access_token: &str, payload: &SharedGeneticPayload) -> Result<(), DatabaseError> {
        let response = self
            .client
            .post(format!("{}/genetic_sharing", self.auth_base))
            .bearer_auth(access_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "MyHeritage upload failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct FamilyTreeDNAClient {
    client: Client,
    client_id: String,
    client_secret: String,
    auth_base: String,
}

impl FamilyTreeDNAClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            client_id,
            client_secret,
            auth_base: "https://api.familytreedna.com".to_string(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("FAMILYTREE_DNA_CLIENT_ID").ok()?;
        let client_secret = std::env::var("FAMILYTREE_DNA_CLIENT_SECRET").ok()?;
        Some(Self::new(client_id, client_secret))
    }
}

#[async_trait]
impl ExternalGeneticService for FamilyTreeDNAClient {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "FamilyTreeDNA token exchange failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, DatabaseError> {
        let response = self
            .client
            .post(format!("{}/oauth2/token", self.auth_base))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "FamilyTreeDNA refresh failed: {}",
                response.status()
            )));
        }

        let token_resp: OAuthToken = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(token_resp)
    }

    async fn fetch_relative_matches(&self, access_token: &str) -> Result<Vec<RelativeMatch>, DatabaseError> {
        let response = self
            .client
            .get(format!("{}/relatives", self.auth_base))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "FamilyTreeDNA relative fetch failed: {}",
                response.status()
            )));
        }

        let matches: Vec<RelativeMatch> = response
            .json()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(matches)
    }

    async fn upload_genetic_payload(&self, access_token: &str, payload: &SharedGeneticPayload) -> Result<(), DatabaseError> {
        let response = self
            .client
            .post(format!("{}/genetic_sharing", self.auth_base))
            .bearer_auth(access_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DatabaseError::QueryFailed(format!(
                "FamilyTreeDNA upload failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

pub struct GeneticDatabaseIntegrator {
    pub twentythree_and_me: TwentyThreeAndMeClient,
    pub ancestry_dna: AncestryDNAClient,
    pub myheritage: MyHeritageClient,
    pub familytree_dna: FamilyTreeDNAClient,
}

impl GeneticDatabaseIntegrator {
    pub fn new(
        twentythree_and_me: TwentyThreeAndMeClient,
        ancestry_dna: AncestryDNAClient,
        myheritage: MyHeritageClient,
        familytree_dna: FamilyTreeDNAClient,
    ) -> Self {
        Self {
            twentythree_and_me,
            ancestry_dna,
            myheritage,
            familytree_dna,
        }
    }

    pub fn from_env() -> Option<Self> {
        Some(Self::new(
            TwentyThreeAndMeClient::from_env()?,
            AncestryDNAClient::from_env()?,
            MyHeritageClient::from_env()?,
            FamilyTreeDNAClient::from_env()?,
        ))
    }

    pub async fn collect_relative_matches(&self, token_map: &HashMap<String, String>) -> Result<Vec<RelativeMatch>, DatabaseError> {
        let mut matches = Vec::new();

        if let Some(token) = token_map.get("23andme") {
            matches.extend(self.twentythree_and_me.fetch_relative_matches(token).await?);
        }
        if let Some(token) = token_map.get("ancestry") {
            matches.extend(self.ancestry_dna.fetch_relative_matches(token).await?);
        }
        if let Some(token) = token_map.get("myheritage") {
            matches.extend(self.myheritage.fetch_relative_matches(token).await?);
        }
        if let Some(token) = token_map.get("familytreedna") {
            matches.extend(self.familytree_dna.fetch_relative_matches(token).await?);
        }

        Ok(matches)
    }

    pub async fn sync_genetic_payload(&self, token_map: &HashMap<String, String>, payload: &SharedGeneticPayload) -> Result<(), DatabaseError> {
        let mut errors = Vec::new();

        if let Some(token) = token_map.get("23andme") {
            if let Err(err) = self.twentythree_and_me.upload_genetic_payload(token, payload).await {
                errors.push(err.to_string());
            }
        }
        if let Some(token) = token_map.get("ancestry") {
            if let Err(err) = self.ancestry_dna.upload_genetic_payload(token, payload).await {
                errors.push(err.to_string());
            }
        }
        if let Some(token) = token_map.get("myheritage") {
            if let Err(err) = self.myheritage.upload_genetic_payload(token, payload).await {
                errors.push(err.to_string());
            }
        }
        if let Some(token) = token_map.get("familytreedna") {
            if let Err(err) = self.familytree_dna.upload_genetic_payload(token, payload).await {
                errors.push(err.to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DatabaseError::QueryFailed(format!(
                "Payload sync errors: {}",
                errors.join("; ")
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_gh_integrator_from_env_not_configured() {
        env::remove_var("TWENTYTHREE_AND_ME_CLIENT_ID");
        env::remove_var("TWENTYTHREE_AND_ME_CLIENT_SECRET");
        env::remove_var("ANCESTRY_DNA_CLIENT_ID");
        env::remove_var("ANCESTRY_DNA_CLIENT_SECRET");
        env::remove_var("MYHERITAGE_CLIENT_ID");
        env::remove_var("MYHERITAGE_CLIENT_SECRET");
        env::remove_var("FAMILYTREE_DNA_CLIENT_ID");
        env::remove_var("FAMILYTREE_DNA_CLIENT_SECRET");

        assert!(GeneticDatabaseIntegrator::from_env().is_none());
    }

    #[test]
    fn test_shared_genetic_payload_roundtrip() {
        let payload = SharedGeneticPayload {
            profile_id: "profile-123".into(),
            snp_data: vec![SNPVariant {
                rsid: "rs429358".into(),
                chromosome: 19,
                position: 45411941,
                genotype: "CT".into(),
                significance: VariantSignificance::Uncertain,
            }],
            ancestry_composition: AncestryBreakdown::default(),
            privacy_level: PrivacyLevel::Protected,
        };

        let json = serde_json::to_string(&payload).expect("serialize payload");
        let decoded: SharedGeneticPayload = serde_json::from_str(&json).expect("deserialize payload");
        assert_eq!(decoded.profile_id, payload.profile_id);
        assert_eq!(decoded.snp_data.len(), 1);
    }
}
