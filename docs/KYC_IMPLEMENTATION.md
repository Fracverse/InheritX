# KYC Verification Flow - Implementation Guide

## Overview

InheritX implements a complete KYC (Know Your Customer) verification flow with secure webhook integration, document upload, and intelligent status tracking. The implementation is provider-agnostic and works with any identity verification service (Sumsub, Persona, Onfido, etc.) that supports webhook callbacks.

## Architecture

```
┌─────────────────┐
│   Frontend      │
│  (React/Next)   │
│                 │
│  - KYC Modal    │
│  - Form Steps   │
│  - Status Views │
└────────┬────────┘
         │
         │ REST API
         │ WebSocket
         │
┌────────▼────────┐
│   Backend       │
│   (Rust/Axum)   │
│                 │
│  - API Routes   │
│  - Webhooks     │
│  - File Upload  │
└────────┬────────┘
         │
         │
    ┌────▼─────┐         ┌──────────────┐
    │PostgreSQL│◄────────┤   Provider   │
    │ Database │         │  (Sumsub/    │
    └──────────┘         │   Persona)   │
                         └──────────────┘
```

## Features

### ✅ Implemented Features

1. **Real-time Status Updates**
   - WebSocket connection for instant status changes
   - Smart polling fallback when WebSocket unavailable
   - Automatic terminal state detection (stops polling)

2. **Secure Document Upload**
   - Multi-file upload support (ID, proof of address, selfie)
   - File validation (5MB max, JPG/PNG/PDF only)
   - Secure storage with user isolation

3. **Webhook Processing**
   - HMAC-SHA256 signature verification
   - Provider-agnostic payload handling
   - Audit logging for compliance
   - Idempotent processing

4. **Rejection & Re-upload Flow**
   - Display rejection reasons to users
   - Conditional re-upload based on `can_resubmit` flag
   - Resubmission tracking and logging

5. **Security**
   - Request authentication via HMAC
   - File type and size validation
   - User-specific document isolation
   - Stale response prevention
   - Audit trail in database

## API Endpoints

### Frontend APIs

#### `GET /api/kyc/status`
Retrieve KYC verification status for authenticated user.

**Query Parameters:**
- `wallet_address` (string) - User's wallet address

**Response:**
```json
{
  "wallet_address": "GDUSER123...",
  "kyc_status": "approved",
  "submitted_at": "2026-11-23T10:30:00Z",
  "approved_at": "2026-11-23T12:00:00Z",
  "rejected_at": null,
  "rejection_reason": null,
  "provider_reference": "sumsub-ref-001",
  "can_resubmit": true
}
```

**Status Values:**
- `pending` - Initial state, no submission yet
- `submitted` - Application submitted, awaiting review
- `approved` - Verification approved
- `rejected` - Verification rejected

---

#### `POST /api/kyc/submit`
Submit KYC application for verification.

**Request Body:**
```json
{
  "wallet_address": "GDUSER123...",
  "full_name": "John Doe",
  "email": "john@example.com",
  "date_of_birth": "1990-01-15",
  "nationality": "US",
  "id_type": "passport",
  "id_number": "P123456789",
  "expiry_date": "2030-12-31",
  "street_address": "123 Main St",
  "city": "New York",
  "country": "US",
  "postal_code": "10001",
  "document_id": "doc_abc123"
}
```

**Response:** Same as `/api/kyc/status`

---

#### `POST /api/kyc/upload`
Upload KYC document (ID, proof of address, selfie).

**Request:** Multipart form data
- `file` (file) - Document file (JPG, PNG, or PDF, max 5MB)
- `document_type` (string) - One of: `id`, `address_proof`, `selfie`

**Response:**
```json
{
  "document_id": "doc_abc123",
  "url": "/uploads/kyc/GDUSER123.../id_document.jpg"
}
```

---

### Provider Webhook API

#### `POST /api/kyc/webhook`
Receive verification status updates from identity provider.

**Headers:**
- `X-KYC-Signature` (required) - HMAC-SHA256 signature
- `Content-Type: application/json`

**Request Body:**
```json
{
  "wallet_address": "GDUSER123...",
  "status": "approved",
  "provider_reference": "sumsub-ref-001",
  "event_type": "kyc.status_update",
  "rejection_reason": "Document expired",
  "can_resubmit": true
}
```

**Response:**
```json
{
  "success": true,
  "message": "KYC status updated to 'approved' for wallet GDUSER123..."
}
```

---

## Configuration

### Backend Environment Variables

```bash
# Required - Webhook signature verification secret
KYC_WEBHOOK_SECRET=your-strong-random-secret-here

# Optional - Document upload directory (default: ./uploads/kyc/)
KYC_UPLOADS_DIR=/var/lib/inheritx/uploads/kyc/

# Database connection
DATABASE_URL=postgres://user:pass@localhost:5432/inheritx
```

### Frontend Environment Variables

```bash
# API base URL
NEXT_PUBLIC_API_URL=http://localhost:8080

# WebSocket endpoint (auto-constructed from API_URL if not set)
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws/kyc

# Disable mock data (should always be false in production)
NEXT_PUBLIC_USE_MOCK_DATA=false
```

---

## Provider Integration

### Sumsub Configuration

1. **Webhook URL**: `https://your-domain.com/api/kyc/webhook`
2. **Secret Key**: Set in Sumsub dashboard, copy to `KYC_WEBHOOK_SECRET`
3. **Events to subscribe**:
   - `applicantReviewed`
   - `applicantReset`
   - `applicantOnHold`

**Payload Mapping:**
```javascript
{
  wallet_address: applicant.externalUserId,
  status: reviewStatus.reviewAnswer.toLowerCase(), // GREEN → approved, RED → rejected
  provider_reference: applicant.id,
  event_type: "kyc.status_update",
  rejection_reason: reviewStatus.rejectLabels?.join(", "),
  can_resubmit: reviewStatus.reviewAnswer !== "RED" || applicant.review.reviewResult.clientComment.includes("resubmit")
}
```

### Persona Configuration

1. **Webhook URL**: `https://your-domain.com/api/kyc/webhook`
2. **Secret Key**: Found in Persona dashboard under Webhooks
3. **Events to subscribe**:
   - `inquiry.completed`
   - `inquiry.failed`
   - `inquiry.expired`

**Payload Mapping:**
```javascript
{
  wallet_address: inquiry.referenceId,
  status: inquiry.status === "completed" ? "approved" : "rejected",
  provider_reference: inquiry.id,
  event_type: `kyc.${inquiry.status}`,
  rejection_reason: inquiry.failureReason,
  can_resubmit: inquiry.status !== "failed" || inquiry.canRetry
}
```

---

## Testing

### Manual Testing with Script

Use the provided test script to simulate webhook events:

```bash
# Test approval
python scripts/test_kyc_webhook.py \
  --wallet GDTEST123 \
  --status approved

# Test rejection with re-upload allowed
python scripts/test_kyc_webhook.py \
  --wallet GDTEST123 \
  --status rejected \
  --reason "Document is expired"

# Test final rejection (no re-upload)
python scripts/test_kyc_webhook.py \
  --wallet GDTEST123 \
  --status rejected \
  --reason "Fraud detected" \
  --no-resubmit
```

### Unit Tests

**Backend:**
```bash
cd backend
cargo test kyc
```

**Frontend:**
```bash
cd frontend
npm test -- KYCVerificationModal
```

### Integration Testing Flow

1. **User submits KYC**:
   ```bash
   curl -X POST http://localhost:8080/api/kyc/submit \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "GDTEST123",
       "full_name": "Test User",
       ...
     }'
   ```

2. **Check status changed to "submitted"**:
   ```bash
   curl http://localhost:8080/api/kyc/status?wallet_address=GDTEST123
   ```

3. **Simulate provider webhook**:
   ```bash
   python scripts/test_kyc_webhook.py --wallet GDTEST123 --status approved
   ```

4. **Verify status changed to "approved"**:
   ```bash
   curl http://localhost:8080/api/kyc/status?wallet_address=GDTEST123
   ```

5. **Check webhook logs**:
   ```sql
   SELECT * FROM kyc_webhook_logs
   WHERE wallet_address = 'GDTEST123'
   ORDER BY processed_at DESC;
   ```

---

## Database Schema

### Tables Used

**`users` table** - Main user records
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    wallet_address TEXT NOT NULL UNIQUE,
    kyc_status kyc_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`kyc_records` table** - KYC submission data
```sql
CREATE TABLE kyc_records (
    id UUID PRIMARY KEY,
    wallet_address TEXT NOT NULL UNIQUE,
    full_name TEXT NOT NULL,
    date_of_birth TEXT NOT NULL,
    street_address TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    document_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`kyc_webhook_logs` table** - Audit trail
```sql
CREATE TABLE kyc_webhook_logs (
    id UUID PRIMARY KEY,
    wallet_address TEXT NOT NULL,
    provider_reference TEXT,
    event_type TEXT NOT NULL,
    kyc_status kyc_status NOT NULL,
    raw_payload JSONB NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL,
    error_message TEXT
);
```

---

## Troubleshooting

### Webhook Returns 503 "Webhook verification is not configured"

**Cause:** `KYC_WEBHOOK_SECRET` environment variable is not set.

**Solution:**
```bash
export KYC_WEBHOOK_SECRET="your-secret-here"
```

### Webhook Returns 401 "Invalid webhook signature"

**Causes:**
1. Secret mismatch between provider and backend
2. Incorrect signature generation algorithm
3. Body modified after signing

**Solution:**
1. Verify secret matches in both provider dashboard and backend env
2. Ensure HMAC-SHA256 is used with raw request body
3. Check body serialization (no extra whitespace, consistent field order)

**Debug:**
```python
# Generate expected signature
import hmac, hashlib
secret = "your-secret"
body = '{"wallet_address":"GDTEST","status":"approved"}'
expected = hmac.new(secret.encode(), body.encode(), hashlib.sha256).hexdigest()
print(f"Expected: sha256={expected}")
```

### Status Not Updating in Frontend

**Causes:**
1. WebSocket connection failed
2. Polling disabled
3. Terminal state reached (approved/rejected) - polling stops

**Solution:**
1. Check browser console for WebSocket errors
2. Verify `NEXT_PUBLIC_WS_URL` or `NEXT_PUBLIC_API_URL` is correct
3. For terminal states, this is expected behavior

**Debug:**
```javascript
// Check WebSocket connection in browser console
const ws = new WebSocket('ws://localhost:8080/ws/kyc');
ws.onopen = () => console.log('Connected');
ws.onmessage = (msg) => console.log('Received:', msg.data);
ws.onerror = (err) => console.error('Error:', err);
```

### Document Upload Fails

**Causes:**
1. File too large (>5MB)
2. Invalid file type (not JPG/PNG/PDF)
3. Upload directory not writable

**Solution:**
1. Compress or resize images
2. Convert documents to supported formats
3. Check filesystem permissions:
   ```bash
   mkdir -p ./uploads/kyc
   chmod 755 ./uploads/kyc
   ```

### "Try Again" Button Not Appearing for Rejection

**Cause:** Webhook payload doesn't include `can_resubmit: true` or sets it to `false`.

**Solution:**
Ensure provider webhook includes:
```json
{
  "status": "rejected",
  "can_resubmit": true
}
```

If field is missing, it defaults to `true` in backend.

---

## Security Best Practices

### Production Checklist

- [ ] Use strong random `KYC_WEBHOOK_SECRET` (min 32 characters)
- [ ] Enable HTTPS for webhook endpoint (required by most providers)
- [ ] Rotate webhook secret periodically
- [ ] Monitor `kyc_webhook_logs` for failed attempts
- [ ] Set up alerts for repeated signature failures
- [ ] Use S3/object storage instead of local filesystem for uploads
- [ ] Implement rate limiting on upload endpoint
- [ ] Review uploaded files for malware before processing
- [ ] Encrypt sensitive data in `kyc_records` table
- [ ] Regular database backups including audit logs
- [ ] GDPR/CCPA compliance for data retention

### Webhook Signature Verification

The backend uses constant-time comparison to prevent timing attacks:

```rust
mac.verify_slice(&expected)  // Constant-time HMAC verification
```

This prevents attackers from using timing differences to guess valid signatures.

---

## Performance Considerations

### Polling Strategy

- **Active polling**: Every 5 seconds when status is `submitted`
- **Inactive**: Stops immediately when status reaches `approved` or `rejected`
- **WebSocket preference**: Polling pauses when WebSocket is healthy

### Database Optimization

Indexes on frequently queried columns:
```sql
CREATE INDEX idx_users_wallet ON users(wallet_address);
CREATE INDEX idx_kyc_records_wallet ON kyc_records(wallet_address);
CREATE INDEX idx_webhook_logs_wallet ON kyc_webhook_logs(wallet_address);
CREATE INDEX idx_webhook_logs_time ON kyc_webhook_logs(processed_at DESC);
```

### File Storage

For production, migrate to object storage:
- AWS S3
- Cloudflare R2
- MinIO (self-hosted S3-compatible)
- Google Cloud Storage

---

## Support & Maintenance

### Monitoring Metrics

Track these metrics in production:

1. **Webhook health**:
   - Success rate from `kyc_webhook_logs`
   - Average processing time
   - Failed signature attempts

2. **User flow**:
   - Submission rate
   - Approval rate
   - Rejection rate
   - Re-submission rate after rejection

3. **System health**:
   - WebSocket connection count
   - API response times
   - Upload success rate

### Log Analysis

```sql
-- Daily KYC submission stats
SELECT
    DATE(created_at) as date,
    COUNT(*) as submissions,
    SUM(CASE WHEN kyc_status = 'approved' THEN 1 ELSE 0 END) as approvals,
    SUM(CASE WHEN kyc_status = 'rejected' THEN 1 ELSE 0 END) as rejections
FROM kyc_records
WHERE created_at >= NOW() - INTERVAL '30 days'
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- Webhook processing health
SELECT
    DATE(processed_at) as date,
    COUNT(*) as total_webhooks,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful,
    SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed
FROM kyc_webhook_logs
WHERE processed_at >= NOW() - INTERVAL '7 days'
GROUP BY DATE(processed_at)
ORDER BY date DESC;
```

---

## License & Credits

**Implementation**: InheritX Engineering Team
**Issue**: #1073
**Branch**: `feat/1073-kyc-document-verification`
**Date**: November 2026

---

For questions or issues, please open a GitHub issue or contact the development team.
