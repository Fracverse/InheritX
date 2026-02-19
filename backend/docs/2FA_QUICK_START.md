# 2FA Quick Start Guide

## Prerequisites
- PostgreSQL database running
- Rust toolchain installed
- C compiler (gcc/clang) installed

## Setup Steps

### 1. Install System Dependencies
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# Fedora/RHEL
sudo dnf install gcc openssl-devel
```

### 2. Configure Environment
Copy and update the environment file:
```bash
cp env.example .env
```

Edit `.env` and set:
```bash
DATABASE_URL=postgres://user:password@localhost/inheritx
PORT=8080

# Email Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
FROM_EMAIL=noreply@inheritx.com
```

### 3. Run Database Migrations
```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

### 4. Build and Run
```bash
cargo build --release
cargo run
```

## Testing the 2FA Endpoints

### 1. Create a Test User
```bash
psql -d inheritx -c "
INSERT INTO users (id, email, password_hash, wallet_address) 
VALUES (
  '00000000-0000-0000-0000-000000000001',
  'test@example.com',
  '\$2b\$12\$dummy_hash',
  '0x1234567890abcdef'
);
"
```

### 2. Send OTP
```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001"
  }'
```

Expected response:
```json
{
  "success": true,
  "message": "OTP sent successfully to your email"
}
```

Check the server logs for the OTP (since email is not configured yet):
```
INFO Sending OTP to test@example.com: 123456 (expires in 5 minutes)
```

### 3. Verify OTP
```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

Expected response:
```json
{
  "success": true,
  "message": "OTP verified successfully"
}
```

## Common Issues

### Issue: "User not found"
**Solution:** Ensure the user exists in the database with the provided UUID.

### Issue: "OTP has expired"
**Solution:** Request a new OTP. OTPs expire after 5 minutes.

### Issue: "Too many attempts"
**Solution:** Request a new OTP. You get 3 attempts per OTP.

### Issue: "Invalid OTP format"
**Solution:** Ensure OTP is exactly 6 digits.

### Issue: Email not sending
**Solution:** 
1. Check SMTP credentials in `.env`
2. For Gmail, use an App Password (not your regular password)
3. Check server logs for email errors

## Integration with Plan Creation

```rust
// Example: Protect plan creation with 2FA
async fn create_plan_with_2fa(
    user_id: Uuid,
    otp: String,
    plan_data: CreatePlanRequest,
) -> Result<Plan, ApiError> {
    // 1. Verify 2FA
    let verify_response = verify_2fa(user_id, otp).await?;
    
    if !verify_response.success {
        return Err(ApiError::Unauthorized);
    }
    
    // 2. Create plan
    let plan = create_plan(user_id, plan_data).await?;
    
    Ok(plan)
}
```

## Integration with Claim Process

```rust
// Example: Protect claim with 2FA
async fn claim_plan_with_2fa(
    user_id: Uuid,
    plan_id: Uuid,
    otp: String,
) -> Result<ClaimResponse, ApiError> {
    // 1. Verify 2FA
    let verify_response = verify_2fa(user_id, otp).await?;
    
    if !verify_response.success {
        return Err(ApiError::Unauthorized);
    }
    
    // 2. Process claim
    let claim = process_claim(user_id, plan_id).await?;
    
    Ok(claim)
}
```

## Monitoring

### Check Active OTPs
```sql
SELECT 
    u.email,
    t.expires_at,
    t.attempts,
    CASE 
        WHEN t.expires_at < NOW() THEN 'expired'
        WHEN t.attempts >= 3 THEN 'max_attempts'
        ELSE 'active'
    END as status
FROM user_2fa t
JOIN users u ON t.user_id = u.id;
```

### Cleanup Expired OTPs
```sql
DELETE FROM user_2fa WHERE expires_at < NOW();
```

Or use the built-in function:
```rust
use inheritx_backend::two_fa::cleanup_expired_otps;

let deleted = cleanup_expired_otps(&pool).await?;
println!("Cleaned up {} expired OTPs", deleted);
```

## Production Checklist

- [ ] Configure real SMTP service (SendGrid, AWS SES, etc.)
- [ ] Set up rate limiting on 2FA endpoints
- [ ] Enable HTTPS/TLS
- [ ] Set up monitoring and alerting
- [ ] Configure log aggregation
- [ ] Set up periodic OTP cleanup job
- [ ] Test email delivery
- [ ] Configure proper CORS settings
- [ ] Set up database backups
- [ ] Review security settings

## Next Steps

1. Integrate with frontend
2. Add rate limiting
3. Implement audit logging
4. Add SMS as alternative delivery method
5. Consider TOTP for enhanced security
