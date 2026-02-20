# 2FA Quick Start Guide

## Setup

1. **Run the migration**:
```bash
cd InheritX/backend
sqlx migrate run
```

2. **Start the server**:
```bash
cargo run
```

## Usage Flow

### Step 1: User requests OTP
```bash
POST /user/send-2fa
{
  "user_id": "uuid-here"
}
```

**What happens**:
- System generates 6-digit OTP
- OTP is hashed and stored in database
- Email sent to user's registered email
- OTP expires in 5 minutes

### Step 2: User submits OTP
```bash
POST /user/verify-2fa
{
  "user_id": "uuid-here",
  "otp": "123456"
}
```

**What happens**:
- System checks if OTP exists and hasn't expired
- Verifies OTP against stored hash
- Increments attempt counter if invalid
- Deletes OTP after successful verification or max attempts

## Error Scenarios

| Error | Status | Message |
|-------|--------|---------|
| User not found | 404 | "User not found" |
| No OTP found | 400 | "No OTP found for this user" |
| OTP expired | 400 | "OTP has expired" |
| Invalid OTP | 400 | "Invalid OTP" |
| Max attempts | 400 | "Maximum verification attempts exceeded" |

## Integration Example

```rust
// In your plan creation handler
async fn create_plan_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<PlanResponse>, ApiError> {
    // First, verify 2FA
    let verify_request = Verify2FARequest {
        user_id: payload.user_id,
        otp: payload.otp,
    };
    
    TwoFAService::verify_otp_from_db(
        &state.db,
        verify_request.user_id,
        &verify_request.otp
    ).await?;
    
    // If verification succeeds, proceed with plan creation
    // ... rest of your logic
}
```

## Testing

Use the provided test script:
```bash
cd InheritX/backend
./examples/test_2fa.sh
```

Or test manually with curl (see examples in 2FA_IMPLEMENTATION.md).

## Email Configuration

Currently logs OTP to console. To enable real emails:

1. Choose an email provider (SendGrid, AWS SES, Mailgun)
2. Add credentials to `.env`
3. Update `src/email_service.rs` with provider implementation

## Maintenance

Set up a cron job to clean expired OTPs:
```rust
// Run every hour
TwoFAService::cleanup_expired_otps(&db_pool).await?;
```
