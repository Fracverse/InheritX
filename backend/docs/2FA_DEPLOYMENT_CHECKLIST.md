# 2FA Deployment Checklist

## Pre-Deployment

### 1. Database Setup
- [ ] Run migration: `sqlx migrate run`
- [ ] Verify `user_2fa` table exists
- [ ] Check indexes are created
- [ ] Ensure `users` table has `email` column

### 2. Email Service Configuration
- [ ] Choose email provider (SendGrid/AWS SES/Mailgun)
- [ ] Obtain API credentials
- [ ] Add credentials to `.env` file
- [ ] Update `src/email_service.rs` with provider implementation
- [ ] Test email sending in development

### 3. Environment Variables
Add to `.env`:
```env
DATABASE_URL=postgresql://user:password@host/database
PORT=8080
# Email provider credentials
EMAIL_API_KEY=your_api_key_here
EMAIL_FROM=noreply@inheritx.com
```

### 4. Code Review
- [ ] Review security implementation
- [ ] Check error handling
- [ ] Verify OTP expiration logic
- [ ] Confirm attempt limiting works
- [ ] Test cleanup function

## Testing

### 5. Unit Tests
- [ ] Test OTP generation
- [ ] Test OTP hashing
- [ ] Test OTP verification
- [ ] Test expiration logic
- [ ] Test attempt limiting

### 6. Integration Tests
- [ ] Test send-2fa endpoint
- [ ] Test verify-2fa endpoint
- [ ] Test with invalid user_id
- [ ] Test with expired OTP
- [ ] Test with invalid OTP
- [ ] Test max attempts exceeded
- [ ] Test email delivery

### 7. Load Testing
- [ ] Test concurrent OTP requests
- [ ] Test database performance
- [ ] Monitor response times
- [ ] Check for race conditions

## Deployment

### 8. Production Setup
- [ ] Deploy database migration
- [ ] Update production environment variables
- [ ] Configure email service in production
- [ ] Set up monitoring/logging
- [ ] Configure rate limiting (optional)

### 9. Monitoring
- [ ] Set up alerts for failed OTP sends
- [ ] Monitor OTP verification success rate
- [ ] Track expired OTP cleanup
- [ ] Log suspicious activity (multiple failed attempts)

### 10. Maintenance
- [ ] Set up cron job for OTP cleanup
  ```bash
  # Run every hour
  0 * * * * /path/to/cleanup_script.sh
  ```
- [ ] Monitor database growth
- [ ] Review logs regularly
- [ ] Update documentation as needed

## Security Checklist

### 11. Security Verification
- [ ] OTPs are hashed (not plaintext)
- [ ] OTPs expire after 5 minutes
- [ ] Maximum 3 attempts enforced
- [ ] Used OTPs are deleted
- [ ] Expired OTPs are cleaned up
- [ ] Error messages don't leak information
- [ ] HTTPS enabled in production
- [ ] Rate limiting configured
- [ ] Audit logging enabled

## Integration

### 12. Plan Creation Integration
- [ ] Add 2FA verification to plan creation endpoint
- [ ] Update API documentation
- [ ] Update frontend to request OTP
- [ ] Test end-to-end flow

### 13. Claim Assets Integration
- [ ] Add 2FA verification to claim endpoint
- [ ] Update API documentation
- [ ] Update frontend to request OTP
- [ ] Test end-to-end flow

## Documentation

### 14. User Documentation
- [ ] Document 2FA flow for users
- [ ] Create troubleshooting guide
- [ ] Add FAQ section
- [ ] Update API documentation

### 15. Developer Documentation
- [ ] Update README with 2FA setup
- [ ] Document integration examples
- [ ] Add troubleshooting guide
- [ ] Document maintenance procedures

## Post-Deployment

### 16. Monitoring & Metrics
- [ ] Track OTP send success rate
- [ ] Track OTP verification success rate
- [ ] Monitor average verification time
- [ ] Track failed attempts
- [ ] Monitor email delivery rate

### 17. User Feedback
- [ ] Collect user feedback
- [ ] Monitor support tickets
- [ ] Track common issues
- [ ] Iterate on UX improvements

## Rollback Plan

### 18. Rollback Preparation
- [ ] Document rollback procedure
- [ ] Test rollback in staging
- [ ] Prepare database rollback script
- [ ] Have backup of previous version

## Success Criteria

- [ ] OTP emails delivered within 5 seconds
- [ ] 99%+ OTP verification success rate
- [ ] Zero security incidents
- [ ] Positive user feedback
- [ ] No performance degradation

## Notes

- Keep this checklist updated as requirements change
- Review security practices quarterly
- Update dependencies regularly
- Monitor for new security vulnerabilities
