use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// Integration tests for 2FA endpoints
// Note: These tests require a running database

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test app
    // async fn create_test_app() -> Router {
    //     let config = Config::load().unwrap();
    //     let pool = create_pool(&config.database_url).await.unwrap();
    //     create_app(pool, config).await.unwrap()
    // }

    #[tokio::test]
    #[ignore] // Ignore by default, run with: cargo test -- --ignored
    async fn test_send_2fa_success() {
        // This test requires a real database connection
        // Uncomment and modify as needed for your test environment
        
        // let app = create_test_app().await;
        // let user_id = "00000000-0000-0000-0000-000000000001"; // Test user ID
        
        // let response = app
        //     .oneshot(
        //         Request::builder()
        //             .method("POST")
        //             .uri("/user/send-2fa")
        //             .header("content-type", "application/json")
        //             .body(Body::from(
        //                 json!({
        //                     "user_id": user_id
        //                 })
        //                 .to_string(),
        //             ))
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();
        
        // assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_2fa_user_not_found() {
        // Test with non-existent user
        // let app = create_test_app().await;
        // let user_id = "00000000-0000-0000-0000-000000000999"; // Non-existent user
        
        // let response = app
        //     .oneshot(
        //         Request::builder()
        //             .method("POST")
        //             .uri("/user/send-2fa")
        //             .header("content-type", "application/json")
        //             .body(Body::from(
        //                 json!({
        //                     "user_id": user_id
        //                 })
        //                 .to_string(),
        //             ))
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();
        
        // assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn test_verify_2fa_invalid_format() {
        // Test with invalid OTP format
        // let app = create_test_app().await;
        // let user_id = "00000000-0000-0000-0000-000000000001";
        
        // let response = app
        //     .oneshot(
        //         Request::builder()
        //             .method("POST")
        //             .uri("/user/verify-2fa")
        //             .header("content-type", "application/json")
        //             .body(Body::from(
        //                 json!({
        //                     "user_id": user_id,
        //                     "otp": "12345" // Only 5 digits
        //                 })
        //                 .to_string(),
        //             ))
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();
        
        // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn test_verify_2fa_success() {
        // Full flow test: send OTP, then verify it
        // This requires mocking or using a test email service
        
        // 1. Send OTP
        // 2. Extract OTP from email/logs
        // 3. Verify OTP
        // 4. Assert success
    }

    #[tokio::test]
    #[ignore]
    async fn test_verify_2fa_expired() {
        // Test OTP expiration
        // This requires waiting 5+ minutes or manipulating time
    }

    #[tokio::test]
    #[ignore]
    async fn test_verify_2fa_max_attempts() {
        // Test max attempts limit
        // 1. Send OTP
        // 2. Try to verify with wrong OTP 3 times
        // 3. Assert that 4th attempt fails with "too many attempts" error
    }
}

// Unit tests for 2FA functions
#[cfg(test)]
mod unit_tests {
    use inheritx_backend::two_fa::{generate_otp, hash_otp, verify_otp};

    #[test]
    fn test_generate_otp_format() {
        let otp = generate_otp();
        assert_eq!(otp.len(), 6);
        assert!(otp.chars().all(|c| c.is_ascii_digit()));
        
        // Ensure it's in valid range
        let otp_num: u32 = otp.parse().unwrap();
        assert!(otp_num >= 100000 && otp_num <= 999999);
    }

    #[test]
    fn test_hash_and_verify_otp() {
        let otp = "123456";
        let hash = hash_otp(otp).unwrap();
        
        // Verify correct OTP
        assert!(verify_otp(otp, &hash).unwrap());
        
        // Verify incorrect OTP
        assert!(!verify_otp("654321", &hash).unwrap());
    }

    #[test]
    fn test_hash_otp_different_each_time() {
        let otp = "123456";
        let hash1 = hash_otp(otp).unwrap();
        let hash2 = hash_otp(otp).unwrap();
        
        // Hashes should be different due to salt
        assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        assert!(verify_otp(otp, &hash1).unwrap());
        assert!(verify_otp(otp, &hash2).unwrap());
    }
}
