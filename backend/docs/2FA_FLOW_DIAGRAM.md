# 2FA Flow Diagrams

## 1. Send OTP Flow

```
┌─────────┐                ┌─────────┐                ┌──────────┐                ┌─────────┐
│ Client  │                │ Handler │                │ Database │                │  Email  │
└────┬────┘                └────┬────┘                └────┬─────┘                └────┬────┘
     │                          │                          │                           │
     │ POST /user/send-2fa      │                          │                           │
     │ { user_id }              │                          │                           │
     ├─────────────────────────>│                          │                           │
     │                          │                          │                           │
     │                          │ SELECT email FROM users  │                           │
     │                          │ WHERE id = user_id       │                           │
     │                          ├─────────────────────────>│                           │
     │                          │                          │                           │
     │                          │      email               │                           │
     │                          │<─────────────────────────┤                           │
     │                          │                          │                           │
     │                          │ Generate 6-digit OTP     │                           │
     │                          │ (e.g., 123456)           │                           │
     │                          │                          │                           │
     │                          │ Hash OTP with bcrypt     │                           │
     │                          │ (e.g., $2b$12$...)       │                           │
     │                          │                          │                           │
     │                          │ DELETE old OTPs          │                           │
     │                          ├─────────────────────────>│                           │
     │                          │                          │                           │
     │                          │ INSERT INTO user_2fa     │                           │
     │                          │ (user_id, otp_hash,      │                           │
     │                          │  expires_at, attempts)   │                           │
     │                          ├─────────────────────────>│                           │
     │                          │                          │                           │
     │                          │         OK               │                           │
     │                          │<─────────────────────────┤                           │
     │                          │                          │                           │
     │                          │ Send email with OTP      │                           │
     │                          ├──────────────────────────┼──────────────────────────>│
     │                          │                          │                           │
     │                          │                          │                    Email sent
     │                          │                          │                           │
     │  { success: true,        │                          │                           │
     │    message: "OTP sent" } │                          │                           │
     │<─────────────────────────┤                          │                           │
     │                          │                          │                           │
```

## 2. Verify OTP Flow (Success)

```
┌─────────┐                ┌─────────┐                ┌──────────┐
│ Client  │                │ Handler │                │ Database │
└────┬────┘                └────┬────┘                └────┬─────┘
     │                          │                          │
     │ POST /user/verify-2fa    │                          │
     │ { user_id, otp }         │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │ Validate OTP format      │
     │                          │ (6 digits)               │
     │                          │                          │
     │                          │ SELECT * FROM user_2fa   │
     │                          │ WHERE user_id = ?        │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │  { id, otp_hash,         │
     │                          │    expires_at,           │
     │                          │    attempts }            │
     │                          │<─────────────────────────┤
     │                          │                          │
     │                          │ Check expires_at         │
     │                          │ (< now?)                 │
     │                          │ ✓ Not expired            │
     │                          │                          │
     │                          │ Check attempts           │
     │                          │ (< 3?)                   │
     │                          │ ✓ Attempts OK            │
     │                          │                          │
     │                          │ Verify OTP hash          │
     │                          │ bcrypt::verify()         │
     │                          │ ✓ Valid                  │
     │                          │                          │
     │                          │ DELETE FROM user_2fa     │
     │                          │ WHERE id = ?             │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │         OK               │
     │                          │<─────────────────────────┤
     │                          │                          │
     │  { success: true,        │                          │
     │    message: "Verified" } │                          │
     │<─────────────────────────┤                          │
     │                          │                          │
```

## 3. Verify OTP Flow (Failed - Invalid OTP)

```
┌─────────┐                ┌─────────┐                ┌──────────┐
│ Client  │                │ Handler │                │ Database │
└────┬────┘                └────┬────┘                └────┬─────┘
     │                          │                          │
     │ POST /user/verify-2fa    │                          │
     │ { user_id, otp: "999999" }│                         │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │ SELECT * FROM user_2fa   │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │  { attempts: 0 }         │
     │                          │<─────────────────────────┤
     │                          │                          │
     │                          │ Verify OTP hash          │
     │                          │ ✗ Invalid                │
     │                          │                          │
     │                          │ UPDATE user_2fa          │
     │                          │ SET attempts = 1         │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │         OK               │
     │                          │<─────────────────────────┤
     │                          │                          │
     │  400 Bad Request         │                          │
     │  { error: "Invalid OTP" }│                          │
     │<─────────────────────────┤                          │
     │                          │                          │
```

## 4. Verify OTP Flow (Failed - Expired)

```
┌─────────┐                ┌─────────┐                ┌──────────┐
│ Client  │                │ Handler │                │ Database │
└────┬────┘                └────┬────┘                └────┬─────┘
     │                          │                          │
     │ POST /user/verify-2fa    │                          │
     │ { user_id, otp }         │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │ SELECT * FROM user_2fa   │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │  { expires_at:           │
     │                          │    2024-01-01 10:00 }    │
     │                          │<─────────────────────────┤
     │                          │                          │
     │                          │ Check expires_at         │
     │                          │ (now: 2024-01-01 10:06)  │
     │                          │ ✗ Expired!               │
     │                          │                          │
     │                          │ DELETE FROM user_2fa     │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │         OK               │
     │                          │<─────────────────────────┤
     │                          │                          │
     │  400 Bad Request         │                          │
     │  { error: "OTP expired" }│                          │
     │<─────────────────────────┤                          │
     │                          │                          │
```

## 5. Verify OTP Flow (Failed - Max Attempts)

```
┌─────────┐                ┌─────────┐                ┌──────────┐
│ Client  │                │ Handler │                │ Database │
└────┬────┘                └────┬────┘                └────┬─────┘
     │                          │                          │
     │ POST /user/verify-2fa    │                          │
     │ (4th attempt)            │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │ SELECT * FROM user_2fa   │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │  { attempts: 3 }         │
     │                          │<─────────────────────────┤
     │                          │                          │
     │                          │ Check attempts           │
     │                          │ ✗ attempts >= 3          │
     │                          │                          │
     │                          │ DELETE FROM user_2fa     │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │         OK               │
     │                          │<─────────────────────────┤
     │                          │                          │
     │  400 Bad Request         │                          │
     │  { error: "Too many      │                          │
     │    attempts" }           │                          │
     │<─────────────────────────┤                          │
     │                          │                          │
```

## 6. Complete Plan Creation Flow with 2FA

```
┌─────────┐     ┌──────────┐     ┌─────────┐     ┌──────────┐     ┌─────────┐
│  User   │     │ Frontend │     │ Backend │     │ Database │     │  Email  │
└────┬────┘     └────┬─────┘     └────┬────┘     └────┬─────┘     └────┬────┘
     │               │                │                │                │
     │ Click "Create │                │                │                │
     │ Plan"         │                │                │                │
     ├──────────────>│                │                │                │
     │               │                │                │                │
     │               │ Fill plan      │                │                │
     │               │ details        │                │                │
     │<──────────────┤                │                │                │
     │               │                │                │                │
     │ Submit        │                │                │                │
     ├──────────────>│                │                │                │
     │               │                │                │                │
     │               │ POST /user/    │                │                │
     │               │ send-2fa       │                │                │
     │               ├───────────────>│                │                │
     │               │                │                │                │
     │               │                │ Generate &     │                │
     │               │                │ store OTP      │                │
     │               │                ├───────────────>│                │
     │               │                │                │                │
     │               │                │ Send email     │                │
     │               │                ├────────────────┼───────────────>│
     │               │                │                │                │
     │               │ { success }    │                │                │
     │               │<───────────────┤                │                │
     │               │                │                │                │
     │ Show OTP      │                │                │                │
     │ input         │                │                │                │
     │<──────────────┤                │                │                │
     │               │                │                │         Email arrives
     │               │                │                │                │
     │ Check email   │                │                │                │
     │ (OTP: 123456) │                │                │                │
     │               │                │                │                │
     │ Enter OTP     │                │                │                │
     ├──────────────>│                │                │                │
     │               │                │                │                │
     │               │ POST /user/    │                │                │
     │               │ verify-2fa     │                │                │
     │               ├───────────────>│                │                │
     │               │                │                │                │
     │               │                │ Verify OTP     │                │
     │               │                ├───────────────>│                │
     │               │                │                │                │
     │               │ { success }    │                │                │
     │               │<───────────────┤                │                │
     │               │                │                │                │
     │               │ POST /plans    │                │                │
     │               │ (create plan)  │                │                │
     │               ├───────────────>│                │                │
     │               │                │                │                │
     │               │                │ INSERT plan    │                │
     │               │                ├───────────────>│                │
     │               │                │                │                │
     │               │ { plan }       │                │                │
     │               │<───────────────┤                │                │
     │               │                │                │                │
     │ Plan created! │                │                │                │
     │<──────────────┤                │                │                │
     │               │                │                │                │
```

## 7. Database State Transitions

```
Initial State (No OTP):
┌──────────┐
│ user_2fa │
│  (empty) │
└──────────┘

After Send OTP:
┌────────────────────────────────────────────────┐
│ user_2fa                                       │
├────────────────────────────────────────────────┤
│ id: uuid-1                                     │
│ user_id: user-uuid                             │
│ otp_hash: $2b$12$...                           │
│ expires_at: 2024-01-01 10:05:00 (now + 5 min) │
│ attempts: 0                                    │
│ created_at: 2024-01-01 10:00:00                │
└────────────────────────────────────────────────┘

After Failed Verification (1st attempt):
┌────────────────────────────────────────────────┐
│ user_2fa                                       │
├────────────────────────────────────────────────┤
│ id: uuid-1                                     │
│ user_id: user-uuid                             │
│ otp_hash: $2b$12$...                           │
│ expires_at: 2024-01-01 10:05:00                │
│ attempts: 1  ← Incremented                     │
│ created_at: 2024-01-01 10:00:00                │
└────────────────────────────────────────────────┘

After Successful Verification:
┌──────────┐
│ user_2fa │
│  (empty) │  ← Record deleted
└──────────┘

After Expiration or Max Attempts:
┌──────────┐
│ user_2fa │
│  (empty) │  ← Record deleted
└──────────┘
```

## 8. Error Handling Decision Tree

```
Verify OTP Request
        │
        ├─ OTP format invalid (not 6 digits)?
        │       └─> 400 "Invalid OTP format"
        │
        ├─ No OTP record found?
        │       └─> 400 "No OTP found for user"
        │
        ├─ OTP expired (expires_at < now)?
        │       ├─> Delete record
        │       └─> 400 "OTP has expired"
        │
        ├─ Attempts >= 3?
        │       ├─> Delete record
        │       └─> 400 "Too many attempts"
        │
        ├─ OTP hash doesn't match?
        │       ├─> Increment attempts
        │       └─> 400 "Invalid OTP"
        │
        └─ OTP valid?
                ├─> Delete record
                └─> 200 "OTP verified successfully"
```

## Legend

```
┌─────┐
│     │  = Component/Actor
└─────┘

  ├──>  = Request/Action
  <──┤  = Response

  ✓    = Success/Valid
  ✗    = Failure/Invalid
```
