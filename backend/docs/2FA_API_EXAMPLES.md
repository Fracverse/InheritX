# 2FA API Examples

This document provides practical examples of using the 2FA API endpoints with curl commands and expected responses.

## Prerequisites

- Server running on `http://localhost:8080`
- Test user exists in database with ID: `00000000-0000-0000-0000-000000000001`
- Test user email: `test@example.com`

## 1. Send OTP

### Request

```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001"
  }'
```

### Success Response (200 OK)

```json
{
  "success": true,
  "message": "OTP sent successfully to your email"
}
```

### Server Logs (Development)

```
INFO Sending OTP to test@example.com: 123456 (expires in 5 minutes)
```

### Error Response - User Not Found (404 Not Found)

```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-999999999999"
  }'
```

```json
{
  "error": "Not Found: User not found"
}
```

### Error Response - Invalid UUID Format (400 Bad Request)

```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "invalid-uuid"
  }'
```

```json
{
  "error": "Bad Request: Invalid UUID format"
}
```

## 2. Verify OTP

### Request (Success)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

### Success Response (200 OK)

```json
{
  "success": true,
  "message": "OTP verified successfully"
}
```

### Error Response - Invalid OTP (400 Bad Request)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "999999"
  }'
```

```json
{
  "error": "Bad Request: Invalid OTP"
}
```

### Error Response - Invalid Format (400 Bad Request)

```bash
# Only 5 digits
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "12345"
  }'
```

```json
{
  "error": "Bad Request: Invalid OTP format. Must be 6 digits"
}
```

```bash
# Contains letters
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "abc123"
  }'
```

```json
{
  "error": "Bad Request: Invalid OTP format. Must be 6 digits"
}
```

### Error Response - Expired OTP (400 Bad Request)

```bash
# After waiting 5+ minutes
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

```json
{
  "error": "Bad Request: OTP has expired"
}
```

### Error Response - Too Many Attempts (400 Bad Request)

```bash
# After 3 failed attempts
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "999999"
  }'
```

```json
{
  "error": "Bad Request: Too many attempts. Please request a new OTP"
}
```

### Error Response - No OTP Found (400 Bad Request)

```bash
# Trying to verify without sending OTP first
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

```json
{
  "error": "Bad Request: No OTP found for user"
}
```

## 3. Complete Flow Example

### Step 1: Send OTP

```bash
curl -v -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001"
  }'
```

**Response:**
```
< HTTP/1.1 200 OK
< content-type: application/json
< content-length: 67
< date: Thu, 19 Feb 2026 21:00:00 GMT

{
  "success": true,
  "message": "OTP sent successfully to your email"
}
```

### Step 2: Check Server Logs for OTP (Development Only)

```
2026-02-19T21:00:00.123Z INFO inheritx_backend::email: Sending OTP to test@example.com: 123456 (expires in 5 minutes)
```

### Step 3: Verify OTP

```bash
curl -v -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

**Response:**
```
< HTTP/1.1 200 OK
< content-type: application/json
< content-length: 58
< date: Thu, 19 Feb 2026 21:00:30 GMT

{
  "success": true,
  "message": "OTP verified successfully"
}
```

## 4. Testing Max Attempts

### Attempt 1 (Wrong OTP)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "111111"
  }'
```

**Response:**
```json
{
  "error": "Bad Request: Invalid OTP"
}
```

**Database State:**
```sql
SELECT attempts FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: attempts = 1
```

### Attempt 2 (Wrong OTP)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "222222"
  }'
```

**Response:**
```json
{
  "error": "Bad Request: Invalid OTP"
}
```

**Database State:**
```sql
SELECT attempts FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: attempts = 2
```

### Attempt 3 (Wrong OTP)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "333333"
  }'
```

**Response:**
```json
{
  "error": "Bad Request: Invalid OTP"
}
```

**Database State:**
```sql
SELECT attempts FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: attempts = 3
```

### Attempt 4 (Any OTP)

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

**Response:**
```json
{
  "error": "Bad Request: Too many attempts. Please request a new OTP"
}
```

**Database State:**
```sql
SELECT * FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: (empty) - record deleted
```

## 5. Testing Expiration

### Send OTP

```bash
curl -X POST http://localhost:8080/user/send-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001"
  }'
```

**Database State:**
```sql
SELECT expires_at FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: 2026-02-19 21:05:00 (5 minutes from now)
```

### Wait 5+ Minutes

```bash
# Wait or manually update database for testing
psql -d inheritx -c "
UPDATE user_2fa 
SET expires_at = NOW() - INTERVAL '1 minute' 
WHERE user_id = '00000000-0000-0000-0000-000000000001';
"
```

### Try to Verify

```bash
curl -X POST http://localhost:8080/user/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "00000000-0000-0000-0000-000000000001",
    "otp": "123456"
  }'
```

**Response:**
```json
{
  "error": "Bad Request: OTP has expired"
}
```

**Database State:**
```sql
SELECT * FROM user_2fa WHERE user_id = '00000000-0000-0000-0000-000000000001';
-- Result: (empty) - record deleted
```

## 6. Using with JavaScript/TypeScript

### Send OTP

```typescript
async function sendOTP(userId: string): Promise<void> {
  const response = await fetch('http://localhost:8080/user/send-2fa', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ user_id: userId }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }

  const data = await response.json();
  console.log(data.message);
}

// Usage
try {
  await sendOTP('00000000-0000-0000-0000-000000000001');
  console.log('OTP sent successfully');
} catch (error) {
  console.error('Failed to send OTP:', error.message);
}
```

### Verify OTP

```typescript
async function verifyOTP(userId: string, otp: string): Promise<boolean> {
  const response = await fetch('http://localhost:8080/user/verify-2fa', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ user_id: userId, otp }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }

  const data = await response.json();
  return data.success;
}

// Usage
try {
  const isValid = await verifyOTP(
    '00000000-0000-0000-0000-000000000001',
    '123456'
  );
  
  if (isValid) {
    console.log('OTP verified successfully');
    // Proceed with plan creation or claim
  }
} catch (error) {
  console.error('Failed to verify OTP:', error.message);
}
```

### Complete Flow with React

```typescript
import { useState } from 'react';

function TwoFactorAuth({ userId, onSuccess }: Props) {
  const [otp, setOtp] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [otpSent, setOtpSent] = useState(false);

  const handleSendOTP = async () => {
    setLoading(true);
    setError('');
    
    try {
      const response = await fetch('http://localhost:8080/user/send-2fa', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error);
      }

      setOtpSent(true);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleVerifyOTP = async () => {
    setLoading(true);
    setError('');
    
    try {
      const response = await fetch('http://localhost:8080/user/verify-2fa', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId, otp }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error);
      }

      const data = await response.json();
      if (data.success) {
        onSuccess();
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      {!otpSent ? (
        <button onClick={handleSendOTP} disabled={loading}>
          {loading ? 'Sending...' : 'Send OTP'}
        </button>
      ) : (
        <div>
          <input
            type="text"
            value={otp}
            onChange={(e) => setOtp(e.target.value)}
            placeholder="Enter 6-digit OTP"
            maxLength={6}
          />
          <button onClick={handleVerifyOTP} disabled={loading || otp.length !== 6}>
            {loading ? 'Verifying...' : 'Verify OTP'}
          </button>
          <button onClick={handleSendOTP} disabled={loading}>
            Resend OTP
          </button>
        </div>
      )}
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

## 7. Database Queries for Testing

### Check OTP Status

```sql
SELECT 
    u.email,
    t.otp_hash,
    t.expires_at,
    t.attempts,
    t.created_at,
    CASE 
        WHEN t.expires_at < NOW() THEN 'expired'
        WHEN t.attempts >= 3 THEN 'max_attempts'
        ELSE 'active'
    END as status,
    EXTRACT(EPOCH FROM (t.expires_at - NOW())) as seconds_until_expiry
FROM user_2fa t
JOIN users u ON t.user_id = u.id
WHERE u.id = '00000000-0000-0000-0000-000000000001';
```

### Manually Expire OTP (for testing)

```sql
UPDATE user_2fa 
SET expires_at = NOW() - INTERVAL '1 minute' 
WHERE user_id = '00000000-0000-0000-0000-000000000001';
```

### Manually Set Attempts (for testing)

```sql
UPDATE user_2fa 
SET attempts = 2
WHERE user_id = '00000000-0000-0000-0000-000000000001';
```

### Clean Up All OTPs

```sql
DELETE FROM user_2fa;
```

### Clean Up Expired OTPs

```sql
DELETE FROM user_2fa WHERE expires_at < NOW();
```

## 8. Health Check

```bash
curl http://localhost:8080/health
```

**Response:**
```json
{
  "status": "ok",
  "message": "App is healthy"
}
```

```bash
curl http://localhost:8080/health/db
```

**Response:**
```json
{
  "status": "ok",
  "message": "Database is connected"
}
```
