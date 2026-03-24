# Emergency Access Notification System (Issue #293)

## Overview

The Emergency Access Notification System provides administrators with the ability to grant, revoke, and track emergency access to plans while automatically notifying users of access lifecycle events.

## Features

### 1. Grant Emergency Access
Administrators can grant emergency access to a plan with optional expiration.

**Endpoint:** `POST /api/admin/emergency-access/grant`

**Request:**
```json
{
  "plan_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_type": "admin_override",
  "reason": "Risk mitigation - liquidation risk detected",
  "expires_in_hours": 48
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "success": true,
    "access_id": "550e8400-e29b-41d4-a716-446655440001",
    "message": "Emergency access granted successfully"
  }
}
```

**Notifications Sent:**
- User receives `emergency_access_granted` notification with access type, reason, and expiration time

**Audit Log:**
- Action: `emergency_access_granted`
- Entity: Plan
- Admin ID: Recorded

### 2. Revoke Emergency Access
Administrators can revoke active emergency access at any time.

**Endpoint:** `POST /api/admin/emergency-access/revoke`

**Request:**
```json
{
  "access_id": "550e8400-e29b-41d4-a716-446655440001",
  "reason": "Risk resolved - no longer needed"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "success": true,
    "access_id": "550e8400-e29b-41d4-a716-446655440001",
    "message": "Emergency access revoked successfully"
  }
}
```

**Notifications Sent:**
- User receives `emergency_access_revoked` notification with revocation reason

**Audit Log:**
- Action: `emergency_access_revoked`
- Entity: Plan
- Admin ID: Recorded

### 3. Access Expiration Notifications
The system automatically checks for expiring access every hour and sends notifications 24 hours before expiration.

**Background Job:**
- Runs every 60 minutes
- Checks for access expiring within the next 24 hours
- Sends `emergency_access_expiring` notifications to affected users
- Marks expired access as `expired` status

**Notification:**
```
"Emergency access to your plan will expire at 2026-03-26 14:30:00 UTC"
```

### 4. View Emergency Access Records

#### Get All Emergency Access (Admin)
**Endpoint:** `GET /api/admin/emergency-access/all`

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "plan_id": "550e8400-e29b-41d4-a716-446655440000",
      "granted_by": "550e8400-e29b-41d4-a716-446655440002",
      "granted_to": null,
      "access_type": "admin_override",
      "reason": "Risk mitigation",
      "granted_at": "2026-03-24T10:00:00Z",
      "expires_at": "2026-03-26T10:00:00Z",
      "revoked_at": null,
      "revoked_by": null,
      "revocation_reason": null,
      "status": "active",
      "created_at": "2026-03-24T10:00:00Z",
      "updated_at": "2026-03-24T10:00:00Z"
    }
  ],
  "count": 1
}
```

#### Get Emergency Access for Specific Plan
**Endpoint:** `GET /api/admin/emergency-access/plan/:plan_id`

**Response:** Same structure as above, filtered by plan

## Database Schema

### emergency_access Table

```sql
CREATE TABLE emergency_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    granted_by UUID NOT NULL REFERENCES admins(id),
    granted_to UUID REFERENCES users(id) ON DELETE SET NULL,
    access_type VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    granted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoked_by UUID REFERENCES admins(id),
    revocation_reason TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
```

### Indexes
- `idx_emergency_access_plan_id` - For querying by plan
- `idx_emergency_access_granted_to` - For querying by user
- `idx_emergency_access_status` - For filtering by status
- `idx_emergency_access_expires_at` - For expiration checks

## Notification Types

Three new notification types are added to the system:

1. **emergency_access_granted** - Sent when access is granted
2. **emergency_access_revoked** - Sent when access is revoked
3. **emergency_access_expiring** - Sent 24 hours before expiration

## Audit Actions

Three new audit actions are recorded:

1. **emergency_access_granted** - When admin grants access
2. **emergency_access_revoked** - When admin revokes access
3. **emergency_access_expired** - When system marks access as expired

## Background Job

The `EmergencyAccessJobService` runs automatically on server startup and:

1. **Every 60 minutes:**
   - Checks for access expiring within 24 hours
   - Sends notifications to affected users
   - Marks expired access as `expired` status

2. **Error Handling:**
   - Logs warnings if notification creation fails
   - Continues processing other records
   - Does not crash the service

## Access Statuses

- **active** - Access is currently valid
- **expired** - Access has passed its expiration time
- **revoked** - Access was manually revoked by an admin

## Integration Points

### With Risk Engine
Emergency access can be used in conjunction with risk overrides to:
- Pause risky plans
- Override risk monitoring
- Grant temporary admin access

### With Notifications System
- Uses existing `NotificationService` for atomic transactions
- Participates in user's transaction for consistency
- Supports pagination and filtering

### With Audit Logging
- All access changes are logged to `action_logs`
- Includes admin ID, timestamp, and reason
- Supports compliance and audit trails

## Example Workflow

1. **Risk Detected:** Risk engine flags a plan as risky
2. **Admin Grants Access:** Admin grants emergency access with 48-hour expiration
3. **User Notified:** User receives notification of emergency access
4. **24 Hours Later:** System sends expiration warning
5. **48 Hours Later:** Access automatically expires, status updated
6. **Admin Revokes Early:** If needed, admin can revoke before expiration
7. **User Notified:** User receives revocation notification

## Security Considerations

1. **Admin-Only Operations:** All grant/revoke operations require admin authentication
2. **Audit Trail:** All access changes are logged with admin ID
3. **Atomic Transactions:** Notifications and audit logs are atomic with access changes
4. **Expiration Enforcement:** Expired access is automatically marked and can be checked
5. **Reason Tracking:** All access grants and revocations include reasons for compliance

## Testing

Unit tests are included for:
- Serialization/deserialization of access records
- Request/response structures
- Status transitions

Integration tests should verify:
- Notification delivery
- Audit log creation
- Background job execution
- Expiration checking
- Concurrent access handling

## Future Enhancements

1. **Access Levels:** Different types of emergency access (read-only, full control, etc.)
2. **Approval Workflow:** Multi-level approval for emergency access
3. **Time-based Restrictions:** Access only during specific hours
4. **IP Whitelisting:** Restrict access to specific IP addresses
5. **Activity Logging:** Track what actions were taken during emergency access
6. **Escalation:** Automatic escalation if access is used extensively
