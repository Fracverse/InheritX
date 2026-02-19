# Two-Factor Authentication (2FA) Implementation

## Overview
This document describes the implementation of Two-Factor Authentication (2FA) for plan creation and claiming in the InheritX backend.

## Features
- 6-digit OTP generation
- OTP sent via email to user's KYC-verified email
- OTP expires after 5 minutes
- Maximum 3 verification attempts
- OTP stored as bcrypt hash in PostgreSQL
- Automatic cleanup of expired OTPs

## Database Schema

### Table: `user_2fa`
```sql
CREATE TABLE user_2fa (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    otp_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    attempts INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT check_attempts CHECK (attempts >= 0 AND attempts <= 3)
);
```

## API Endpoints

### 1. Send OTP
**Endpoint:** `POST /user/send-2fa`

**Request Body:**
```json
{
  "user_id": "uuid-string"
}
```

**Response (Success):**
```json
{
  "success": true,
  "message": "OTP sent successfully to your email"
}
```

**Response (Error):**
```json
{
  "error": "User not found"
}
```

**Status Codes:**
- `200 OK` - OTP sent successfully
- `404 Not Found` - User not found
- `500 Internal Server Error` - Server error

### 2. Verify OTP
**Endpoint:** `POST /user/verify-2fa`

**Request Body:**
```json
{
  "user_id": "uuid-string",
  "otp": "123456"
}
```

**Response (Success):**
```json
{
  "success": true,
  "message": "OTP verified successfully"
}
```

**Response (Error):**
```json
{
  "error": "Invalid OTP"
}
```

**Status Codes:**
- `200 OK` - OTP verified successfully
- `400 Bad Request` - Invalid OTP, expired OTP, or too many attempts
- `500 Internal Server Error` - Server error

## Error Handling

### Error Types
1. **Invalid OTP Format**
   - Message: "Invalid OTP format. Must be 6 digits"
   - Status: 400 Bad Request

2. **Invalid OTP**
   - Message: "Invalid OTP"
   - Status: 400 Bad Request
   - Increments attempt counter

3. **Expired OTP**
   - Message: "OTP has expired"
   - Status: 400 Bad Request
   - Automatically deletes expired OTP

4. **Too Many Attempts**
   - Message: "Too many attempts. Please request a new OTP"
   - Status: 400 Bad Request
   - Automatically deletes OTP after 3 failed attempts

5. **No OTP Found**
   - Message: "No OTP found for user"
   - Status: 400 Bad Request

6. **User Not Found**
   - Message: "User not found"
   - Status: 404 Not Found

## Security Features

### OTP Generation
- 6-digit random number (100000-999999)
- Generated using cryptographically secure random number generator

### OTP Storage
- OTP is hashed using bcrypt with default cost (12)
- Only hash is stored in database
- Original OTP is never stored

### OTP Expiration
- OTP expires after 5 minutes
- Expired OTPs are automatically deleted on verification attempt
- Cleanup function available for periodic maintenance

### Attempt Limiting
- Maximum 3 verification attempts per OTP
- Attempt counter incremented on failed verification
- OTP deleted after max attempts reached

### Database Constraints
- Foreign key constraint ensures user exists
- Check constraint enforces attempt limit (0-3)
- Cascade delete removes OTPs when user is deleted

## Configuration

### Environment Variables
Add these to your `.env` file:

```bash
# Email Configuration (SMTP)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
FROM_EMAIL=noreply@inheritx.com
```

### Email Service Integration
The current implementation logs OTPs to the console. To integrate with a real email service:

1. **Using SendGrid:**
```rust
// In src/email.rs
use sendgrid::v3::*;

pub async fn send_otp(&self, to_email: &str, otp: &str) -> Result<(), ApiError> {
    let mail = Message::new(Email::new(&self.config.from_email))
        .set_subject("Your InheritX 2FA Code")
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(format!("Your verification code is: <b>{}</b>", otp))
        )
        .add_personalization(
            Personalization::new(Email::new(to_email))
        );

    let sender = Sender::new(self.config.smtp_password.clone());
    sender.send(&mail).await?;
    Ok(())
}
```

2. **Using AWS SES:**
```rust
// Add aws-sdk-sesv2 to Cargo.toml
use aws_sdk_sesv2::Client;

pub async fn send_otp(&self, to_email: &str, otp: &str) -> Result<(), ApiError> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    client.send_email()
        .from_email_address(&self.config.from_email)
        .destination(
            Destination::builder()
                .to_addresses(to_email)
                .build()
        )
        .content(
            EmailContent::builder()
                .simple(
                    Message::builder()
                        .subject(Content::builder().data("Your InheritX 2FA Code").build())
                        .body(Body::builder()
                            .html(Content::builder()
                                .data(format!("Your verification code is: <b>{}</b>", otp))
                                .build())
                            .build())
                        .build()
                )
                .build()
        )
        .send()
        .await?;
    
    Ok(())
}
```

## Usage Example

### Plan Creation Flow
```rust
// 1. User initiates plan creation
// 2. Send OTP
let response = client.post("/user/send-2fa")
    .json(&json!({ "user_id": user_id }))
    .send()
    .await?;

// 3. User receives OTP via email and enters it
// 4. Verify OTP
let response = client.post("/user/verify-2fa")
    .json(&json!({
        "user_id": user_id,
        "otp": "123456"
    }))
    .send()
    .await?;

// 5. If verification succeeds, proceed with plan creation
if response.json::<Verify2faResponse>().await?.success {
    // Create plan
}
```

### Claim Flow
```rust
// 1. User initiates claim
// 2. Send OTP
let response = client.post("/user/send-2fa")
    .json(&json!({ "user_id": user_id }))
    .send()
    .await?;

// 3. User receives OTP via email and enters it
// 4. Verify OTP
let response = client.post("/user/verify-2fa")
    .json(&json!({
        "user_id": user_id,
        "otp": "123456"
    }))
    .send()
    .await?;

// 5. If verification succeeds, proceed with claim
if response.json::<Verify2faResponse>().await?.success {
    // Process claim
}
```

## Testing

### Manual Testing with curl

1. **Send OTP:**
```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{"user_id": "your-user-uuid"}'
```

2. **Verify OTP:**
```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{"user_id": "your-user-uuid", "otp": "123456"}'
```

### Unit Tests
Run tests with:
```bash
cargo test
```

## Maintenance

### Cleanup Expired OTPs
The system automatically deletes expired OTPs during verification attempts. For periodic cleanup, you can call:

```rust
use inheritx_backend::two_fa::cleanup_expired_otps;

// In a scheduled job
let deleted_count = cleanup_expired_otps(&pool).await?;
tracing::info!("Cleaned up {} expired OTPs", deleted_count);
```

## Migration

To apply the database migration:
```bash
cd InheritX/backend
sqlx migrate run
```

## Future Enhancements
- SMS OTP delivery option
- TOTP (Time-based OTP) support
- Backup codes for account recovery
- Rate limiting on OTP requests
- Audit logging for 2FA events
- Multi-channel delivery (email + SMS)
