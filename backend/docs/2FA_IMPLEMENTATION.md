# Two-Factor Authentication (2FA) Implementation

## Overview
This implementation provides Two-Factor Authentication for plan creation and claiming in the InheritX backend. A 6-digit OTP is sent to the user's email (provided during KYC submission) and stored securely in PostgreSQL.

## Features

### Core Requirements ✅
- **Endpoints**:
  - `POST /user/send-2fa` - Send OTP to user's email
  - `POST /user/verify-2fa` - Verify OTP provided by user

- **PostgreSQL Storage**:
  - Table: `user_2fa`
  - Columns: `id`, `user_id`, `otp_hash`, `expires_at`, `attempts`, `created_at`

- **Security Features**:
  - OTP expires after 5 minutes
  - Maximum 3 verification attempts
  - OTP stored as bcrypt hash (not plaintext)
  - Automatic cleanup of expired/used OTPs

- **Error Handling**:
  - Invalid OTP
  - Expired OTP
  - Too many attempts
  - User not found

## API Endpoints

### 1. Send OTP
**Endpoint**: `POST /user/send-2fa`

**Request Body**:
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response** (200 OK):
```json
{
  "message": "OTP sent to user@example.com",
  "expires_in_seconds": 300
}
```

**Error Responses**:
- `404 Not Found`: User not found
- `500 Internal Server Error`: Database or email service error

### 2. Verify OTP
**Endpoint**: `POST /user/verify-2fa`

**Request Body**:
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "otp": "123456"
}
```

**Response** (200 OK):
```json
{
  "message": "OTP verified successfully",
  "verified": true
}
```

**Error Responses**:
- `400 Bad Request`: Invalid OTP, expired OTP, or max attempts exceeded
- `404 Not Found`: No OTP found for user
- `500 Internal Server Error`: Database error

## Database Schema

```sql
CREATE TABLE user_2fa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    otp_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_2fa_user_id ON user_2fa(user_id);
CREATE INDEX idx_user_2fa_expires_at ON user_2fa(expires_at);
```

## Security Considerations

1. **OTP Hashing**: OTPs are hashed using bcrypt before storage
2. **Expiration**: OTPs automatically expire after 5 minutes
3. **Rate Limiting**: Maximum 3 verification attempts per OTP
4. **Cleanup**: Used and expired OTPs are automatically deleted
5. **Single Use**: Each OTP can only be used once successfully

## Email Integration

The current implementation includes a placeholder email service. To integrate with a real email provider:

### Option 1: SendGrid
```toml
# Add to Cargo.toml
sendgrid = "0.18"
```

### Option 2: AWS SES
```toml
# Add to Cargo.toml
aws-sdk-ses = "1.0"
aws-config = "1.0"
```

### Option 3: Mailgun
```toml
# Add to Cargo.toml
mailgun-rs = "0.1"
```

Update `src/email_service.rs` with your chosen provider's implementation.

## Testing

### Manual Testing with curl

1. **Send OTP**:
```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

2. **Verify OTP**:
```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000", "otp": "123456"}'
```

### Integration with Plan Creation

To require 2FA for plan creation, add a verification check:

```rust
// Before creating a plan
let verified = verify_2fa_handler(state, user_id, otp).await?;
if !verified {
    return Err(ApiError::Unauthorized);
}
// Proceed with plan creation
```

## Maintenance

### Cleanup Expired OTPs
A cleanup function is provided to remove expired OTPs:

```rust
use inheritx_backend::two_fa::TwoFAService;

// Call periodically (e.g., via cron job)
TwoFAService::cleanup_expired_otps(&db_pool).await?;
```

Consider setting up a scheduled task to run this cleanup every hour.

## Configuration

Add to your `.env` file:
```env
DATABASE_URL=postgresql://user:password@localhost/inheritx
PORT=8080
```

## Migration

Run the migration to create the `user_2fa` table:
```bash
cd backend
sqlx migrate run
```

## Dependencies Added

- `rand = "0.8"` - For OTP generation

Existing dependencies used:
- `bcrypt` - For OTP hashing
- `sqlx` - For database operations
- `chrono` - For timestamp handling
- `uuid` - For user identification

## Future Enhancements

1. Add SMS-based OTP as an alternative
2. Implement rate limiting per user
3. Add admin dashboard for OTP monitoring
4. Support for backup codes
5. Integration with authenticator apps (TOTP)
