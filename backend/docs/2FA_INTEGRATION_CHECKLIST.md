# 2FA Integration Checklist

## Pre-Deployment Checklist

### System Setup
- [ ] Install C compiler (gcc/clang)
  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential pkg-config libssl-dev
  ```
- [ ] Install Rust toolchain (if not already installed)
- [ ] Install PostgreSQL
- [ ] Install sqlx-cli
  ```bash
  cargo install sqlx-cli --no-default-features --features postgres
  ```

### Database Setup
- [ ] Create database
  ```bash
  createdb inheritx
  ```
- [ ] Configure DATABASE_URL in `.env`
- [ ] Run migrations
  ```bash
  sqlx migrate run
  ```
- [ ] Verify `user_2fa` table exists
  ```sql
  \d user_2fa
  ```

### Configuration
- [ ] Copy `env.example` to `.env`
- [ ] Set `DATABASE_URL`
- [ ] Set `PORT`
- [ ] Set `SMTP_HOST`
- [ ] Set `SMTP_PORT`
- [ ] Set `SMTP_USERNAME`
- [ ] Set `SMTP_PASSWORD`
- [ ] Set `FROM_EMAIL`

### Email Service Setup
- [ ] Choose email provider (Gmail, SendGrid, AWS SES, etc.)
- [ ] Create account/credentials
- [ ] For Gmail: Enable 2FA and create App Password
- [ ] Test SMTP connection
- [ ] Update `src/email.rs` with real implementation (currently logs only)

### Build & Test
- [ ] Build project
  ```bash
  cargo build
  ```
- [ ] Run unit tests
  ```bash
  cargo test
  ```
- [ ] Start server
  ```bash
  cargo run
  ```
- [ ] Test health endpoint
  ```bash
  curl http://localhost:8080/health
  ```

## Integration Testing Checklist

### Test User Setup
- [ ] Create test user in database
  ```sql
  INSERT INTO users (id, email, password_hash, wallet_address) 
  VALUES (
    '00000000-0000-0000-0000-000000000001',
    'test@example.com',
    '$2b$12$dummy_hash',
    '0x1234567890abcdef'
  );
  ```

### Endpoint Testing

#### Send OTP Endpoint
- [ ] Test successful OTP send
  ```bash
  curl -X POST http://localhost:8080/user/send-2fa \
    -H "Content-Type: application/json" \
    -d '{"user_id": "00000000-0000-0000-0000-000000000001"}'
  ```
- [ ] Verify response is 200 OK
- [ ] Check server logs for OTP
- [ ] Verify OTP in database
  ```sql
  SELECT * FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
  ```
- [ ] Test with non-existent user (should return 404)
- [ ] Test with invalid UUID format (should return 400)

#### Verify OTP Endpoint
- [ ] Test successful verification
  ```bash
  curl -X POST http://localhost:8080/user/verify-2fa \
    -H "Content-Type: application/json" \
    -d '{"user_id": "00000000-0000-0000-0000-000000000001", "otp": "123456"}'
  ```
- [ ] Verify response is 200 OK
- [ ] Verify OTP is deleted from database after success
- [ ] Test with wrong OTP (should return 400, increment attempts)
- [ ] Test with expired OTP (wait 5+ minutes or manipulate DB)
- [ ] Test max attempts (try 3 wrong OTPs, 4th should fail)
- [ ] Test with invalid format (5 digits, letters, etc.)
- [ ] Test with no OTP in database

### Error Handling Testing
- [ ] Invalid OTP format: "12345" (5 digits)
- [ ] Invalid OTP format: "abc123" (contains letters)
- [ ] Invalid OTP: "999999" (wrong code)
- [ ] Expired OTP: Wait 5+ minutes after sending
- [ ] Too many attempts: Try 3 wrong codes
- [ ] No OTP found: Verify without sending
- [ ] User not found: Use non-existent UUID

### Security Testing
- [ ] Verify OTP is hashed in database (not plaintext)
- [ ] Verify different OTPs generate different hashes
- [ ] Verify same OTP generates different hashes (salt)
- [ ] Verify OTP expires after 5 minutes
- [ ] Verify max 3 attempts enforced
- [ ] Verify old OTP is deleted when new one is sent
- [ ] Verify OTP is deleted after successful verification

## Frontend Integration Checklist

### Plan Creation Flow
- [ ] Add 2FA step to plan creation form
- [ ] Implement "Send OTP" button
- [ ] Add OTP input field (6 digits)
- [ ] Show countdown timer (5 minutes)
- [ ] Handle "Resend OTP" functionality
- [ ] Validate OTP format on frontend
- [ ] Show appropriate error messages
- [ ] Disable form submission until OTP verified
- [ ] Test complete flow end-to-end

### Claim Flow
- [ ] Add 2FA step to claim process
- [ ] Implement "Send OTP" button
- [ ] Add OTP input field (6 digits)
- [ ] Show countdown timer (5 minutes)
- [ ] Handle "Resend OTP" functionality
- [ ] Validate OTP format on frontend
- [ ] Show appropriate error messages
- [ ] Disable claim submission until OTP verified
- [ ] Test complete flow end-to-end

### UI/UX Considerations
- [ ] Clear instructions for user
- [ ] Visual feedback for OTP sending
- [ ] Visual feedback for OTP verification
- [ ] Error message display
- [ ] Loading states
- [ ] Countdown timer display
- [ ] Resend OTP button (disabled during countdown)
- [ ] Accessibility (ARIA labels, keyboard navigation)

## Production Deployment Checklist

### Email Service
- [ ] Replace mock email service with real implementation
- [ ] Test email delivery in production environment
- [ ] Configure SPF/DKIM/DMARC records
- [ ] Set up email monitoring
- [ ] Configure bounce handling
- [ ] Set up email rate limiting

### Security
- [ ] Enable HTTPS/TLS
- [ ] Configure CORS properly
- [ ] Add rate limiting to 2FA endpoints
  - Limit OTP requests per user per hour
  - Limit verification attempts per IP
- [ ] Set up WAF (Web Application Firewall)
- [ ] Enable request logging
- [ ] Configure security headers
- [ ] Review and update secrets

### Monitoring
- [ ] Set up application monitoring (Datadog, New Relic, etc.)
- [ ] Configure alerts for:
  - High OTP failure rate
  - Email delivery failures
  - Database errors
  - High latency
- [ ] Set up log aggregation (ELK, CloudWatch, etc.)
- [ ] Create dashboard for 2FA metrics
- [ ] Set up uptime monitoring

### Database
- [ ] Configure database backups
- [ ] Set up database monitoring
- [ ] Create indexes (already in migration)
- [ ] Set up periodic cleanup job for expired OTPs
  ```rust
  // Run every hour
  tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(3600));
      loop {
          interval.tick().await;
          if let Err(e) = cleanup_expired_otps(&pool).await {
              tracing::error!("Failed to cleanup expired OTPs: {}", e);
          }
      }
  });
  ```
- [ ] Review connection pool settings

### Performance
- [ ] Load test 2FA endpoints
- [ ] Optimize database queries if needed
- [ ] Configure connection pooling
- [ ] Set up caching if appropriate
- [ ] Review timeout settings

### Documentation
- [ ] Update API documentation
- [ ] Document error codes
- [ ] Create runbook for common issues
- [ ] Document monitoring and alerting
- [ ] Create incident response plan

### Compliance
- [ ] Review data retention policies
- [ ] Ensure GDPR compliance (if applicable)
- [ ] Document security measures
- [ ] Review audit logging requirements
- [ ] Ensure PII handling compliance

## Post-Deployment Checklist

### Verification
- [ ] Test all endpoints in production
- [ ] Verify email delivery
- [ ] Check monitoring dashboards
- [ ] Review logs for errors
- [ ] Test error scenarios
- [ ] Verify database cleanup job running

### User Communication
- [ ] Notify users about 2FA requirement
- [ ] Provide help documentation
- [ ] Set up support channels
- [ ] Monitor user feedback

### Ongoing Maintenance
- [ ] Schedule regular security reviews
- [ ] Monitor 2FA metrics
- [ ] Review and update documentation
- [ ] Plan for future enhancements
- [ ] Keep dependencies updated

## Rollback Plan

In case of issues:
- [ ] Document rollback procedure
- [ ] Keep previous version deployable
- [ ] Have database rollback script ready
  ```sql
  -- Rollback migration if needed
  DROP TABLE IF EXISTS user_2fa;
  ```
- [ ] Test rollback procedure
- [ ] Document communication plan for rollback

## Success Metrics

Track these metrics post-deployment:
- [ ] OTP delivery success rate
- [ ] OTP verification success rate
- [ ] Average time to verify OTP
- [ ] Number of expired OTPs
- [ ] Number of max attempts reached
- [ ] User complaints/support tickets
- [ ] System uptime
- [ ] API response times

## Notes

- Keep this checklist updated as requirements change
- Review checklist before each deployment
- Document any deviations from checklist
- Share lessons learned with team
