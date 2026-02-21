#[cfg(test)]
mod tests {
    use super::super::service::TwoFAService;

    #[test]
    fn test_generate_otp() {
        let otp = TwoFAService::generate_otp();
        assert_eq!(otp.len(), 6);
        assert!(otp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_hash_and_verify_otp() {
        let otp = "123456";
        let hash = TwoFAService::hash_otp(otp).expect("Failed to hash OTP");

        // Verify correct OTP
        let is_valid = TwoFAService::verify_otp(otp, &hash).expect("Failed to verify OTP");
        assert!(is_valid);

        // Verify incorrect OTP
        let is_invalid = TwoFAService::verify_otp("654321", &hash).expect("Failed to verify OTP");
        assert!(!is_invalid);
    }

    #[test]
    fn test_otp_format() {
        for _ in 0..100 {
            let otp = TwoFAService::generate_otp();
            let num: u32 = otp.parse().expect("OTP should be a valid number");
            assert!(num < 1_000_000, "OTP should be less than 1,000,000");
        }
    }
}
