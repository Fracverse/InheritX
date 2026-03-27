# Legal Will Audit Logs API

## Overview

The Legal Will Audit Logs system provides comprehensive tracking of all actions related to legal will documents. This ensures full transparency, helps resolve disputes, and improves system observability for both users and administrators.

## Features

- **Complete Audit Trail**: All document lifecycle events are logged
- **Queryable API**: Flexible filtering and search capabilities
- **Admin Dashboard**: Statistics and analytics for system monitoring
- **User Activity Tracking**: Users can view their own document activity
- **Security Auditing**: IP address and user agent tracking
- **No Sensitive Data Exposure**: Proper access controls and data sanitization

## Logged Events

The system automatically logs the following events:

### Document Lifecycle

- `will_created` - Document generation
- `will_updated` - Document modifications
- `will_finalized` - Document finalization
- `will_verified` - Document integrity verification

### Signatures

- `will_signed` - Owner signature
- `witness_signed` - Witness signature
- `witness_invited` - Witness invitation
- `witness_declined` - Witness declination

### Security & Access

- `will_encrypted` - Document encryption
- `will_decrypted` - Document download/access
- `will_backup_created` - Backup creation

## Event Data Structure

Each audit log entry contains:

```json
{
  "id": "uuid",
  "event_type": "will_created",
  "document_id": "uuid",
  "plan_id": "uuid",
  "vault_id": "string",
  "user_id": "uuid",
  "event_data": {
    // Event-specific data (varies by event type)
  },
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## API Endpoints

### Admin Endpoints

#### 1. Get Audit Logs (Filtered)

Get audit logs with flexible filtering options.

**Endpoint**: `GET /api/admin/will/audit/logs`

**Authentication**: Admin only

**Query Parameters**:

- `document_id` (UUID, optional): Filter by document
- `plan_id` (UUID, optional): Filter by plan
- `vault_id` (string, optional): Filter by vault
- `user_id` (UUID, optional): Filter by user
- `event_type` (string, optional): Filter by event type
- `start_date` (ISO 8601, optional): Filter by start date
- `end_date` (ISO 8601, optional): Filter by end date
- `limit` (integer, optional): Max results (default: 100, max: 1000)
- `offset` (integer, optional): Pagination offset (default: 0)

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/admin/will/audit/logs?event_type=will_created&limit=50' \
  -H 'Authorization: Bearer <admin_token>'
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "event_type": "will_created",
      "document_id": "...",
      "plan_id": "...",
      "vault_id": "vault-001",
      "user_id": "...",
      "event_data": { ... },
      "ip_address": "192.168.1.1",
      "user_agent": "...",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "count": 50
}
```

#### 2. Get Audit Statistics

Get system-wide audit statistics for admin dashboard.

**Endpoint**: `GET /api/admin/will/audit/statistics`

**Authentication**: Admin only

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/admin/will/audit/statistics' \
  -H 'Authorization: Bearer <admin_token>'
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "total_events": 15234,
    "unique_plans": 1523,
    "unique_users": 892,
    "first_event": "2024-01-01T00:00:00Z",
    "last_event": "2024-03-26T12:00:00Z",
    "event_type_distribution": [
      {
        "event_type": "will_created",
        "count": 1523
      },
      {
        "event_type": "will_signed",
        "count": 1234
      }
    ]
  }
}
```

#### 3. Get Event Types

Get list of all event types in the system.

**Endpoint**: `GET /api/admin/will/audit/event-types`

**Authentication**: Admin only

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/admin/will/audit/event-types' \
  -H 'Authorization: Bearer <admin_token>'
```

**Response**:

```json
{
  "status": "success",
  "data": [
    "will_created",
    "will_updated",
    "will_finalized",
    "will_signed",
    "witness_signed",
    "will_encrypted",
    "will_decrypted",
    "will_backup_created",
    "will_verified",
    "witness_invited",
    "witness_declined"
  ],
  "count": 11
}
```

#### 4. Search Audit Logs

Search audit logs by text content in event data.

**Endpoint**: `GET /api/admin/will/audit/search`

**Authentication**: Admin only

**Query Parameters**:

- `q` (string, required): Search term
- `limit` (integer, optional): Max results (default: 100, max: 1000)

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/admin/will/audit/search?q=formal&limit=20' \
  -H 'Authorization: Bearer <admin_token>'
```

#### 5. Get User Activity

Get audit activity summary for a specific user.

**Endpoint**: `GET /api/admin/will/audit/user/:user_id`

**Authentication**: Admin only

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/admin/will/audit/user/550e8400-e29b-41d4-a716-446655440000' \
  -H 'Authorization: Bearer <admin_token>'
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "total_actions": 42,
    "documents_created": 5,
    "documents_updated": 3,
    "documents_signed": 5,
    "documents_downloaded": 12,
    "first_activity": "2024-01-01T00:00:00Z",
    "last_activity": "2024-03-26T12:00:00Z"
  }
}
```

### User Endpoints

#### 6. Get Plan Audit Summary

Get audit summary for a specific plan (user must own the plan).

**Endpoint**: `GET /api/will/audit/plan/:plan_id/summary`

**Authentication**: User (plan owner)

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/will/audit/plan/550e8400-e29b-41d4-a716-446655440001/summary' \
  -H 'Authorization: Bearer <user_token>'
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "total_events": 15,
    "event_type_counts": [
      {
        "event_type": "will_created",
        "count": 2
      },
      {
        "event_type": "will_signed",
        "count": 2
      },
      {
        "event_type": "will_decrypted",
        "count": 5
      }
    ],
    "recent_events": [
      {
        "id": "...",
        "event_type": "will_decrypted",
        "document_id": "...",
        "plan_id": "...",
        "vault_id": "vault-001",
        "user_id": "...",
        "event_data": { ... },
        "created_at": "2024-03-26T12:00:00Z"
      }
    ],
    "first_event_at": "2024-01-01T00:00:00Z",
    "last_event_at": "2024-03-26T12:00:00Z"
  }
}
```

#### 7. Get My Activity

Get audit activity summary for the authenticated user.

**Endpoint**: `GET /api/will/audit/my-activity`

**Authentication**: User

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/will/audit/my-activity' \
  -H 'Authorization: Bearer <user_token>'
```

**Response**: Same as "Get User Activity" endpoint

## Security Features

### Access Control

1. **Admin Endpoints**: Only accessible by authenticated administrators
2. **User Endpoints**: Users can only access their own data
3. **Plan Ownership**: Plan-specific queries verify ownership
4. **No Sensitive Data**: Passwords, private keys, and sensitive PII are never logged

### Data Tracking

- **User ID**: Who performed the action
- **IP Address**: Where the action originated
- **User Agent**: What client was used
- **Timestamp**: When the action occurred

### Privacy Considerations

- IP addresses and user agents are optional and can be disabled
- Event data is sanitized to remove sensitive information
- Access logs are separate from audit logs

## Use Cases

### 1. Compliance & Legal

```javascript
// Get all actions for a specific document for legal review
const response = await fetch(
  `/api/admin/will/audit/logs?document_id=${documentId}`,
  {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
  },
);
const auditTrail = await response.json();
```

### 2. Security Monitoring

```javascript
// Monitor for suspicious download activity
const response = await fetch(
  `/api/admin/will/audit/logs?event_type=will_decrypted&limit=100`,
  {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
  },
);
const downloads = await response.json();

// Check for unusual patterns
const suspiciousActivity = downloads.data.filter((log) => {
  // Detect multiple downloads from different IPs
  // Detect downloads outside business hours
  // etc.
});
```

### 3. User Activity Dashboard

```javascript
// Show user their document activity
const response = await fetch("/api/will/audit/my-activity", {
  headers: {
    Authorization: `Bearer ${userToken}`,
  },
});
const activity = await response.json();

// Display activity summary
console.log(`Total actions: ${activity.data.total_actions}`);
console.log(`Documents created: ${activity.data.documents_created}`);
console.log(`Last activity: ${activity.data.last_activity}`);
```

### 4. Admin Dashboard

```javascript
// Get system-wide statistics
const response = await fetch("/api/admin/will/audit/statistics", {
  headers: {
    Authorization: `Bearer ${adminToken}`,
  },
});
const stats = await response.json();

// Display dashboard metrics
console.log(`Total events: ${stats.data.total_events}`);
console.log(`Active users: ${stats.data.unique_users}`);
console.log(`Active plans: ${stats.data.unique_plans}`);
```

## Database Schema

### will_event_log Table

```sql
CREATE TABLE will_event_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      VARCHAR(50) NOT NULL,
    document_id     UUID NOT NULL,
    plan_id         UUID NOT NULL,
    vault_id        VARCHAR(255) NOT NULL,
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    event_data      JSONB NOT NULL,
    ip_address      INET,
    user_agent      TEXT,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_will_event_log_document_id ON will_event_log(document_id);
CREATE INDEX idx_will_event_log_plan_id ON will_event_log(plan_id);
CREATE INDEX idx_will_event_log_vault_id ON will_event_log(vault_id);
CREATE INDEX idx_will_event_log_user_id ON will_event_log(user_id);
CREATE INDEX idx_will_event_log_event_type ON will_event_log(event_type);
CREATE INDEX idx_will_event_log_created_at ON will_event_log(created_at DESC);
CREATE INDEX idx_will_event_log_event_data ON will_event_log USING GIN (event_data);
```

## Performance Considerations

1. **Indexes**: Comprehensive indexes for fast queries
2. **Pagination**: All list endpoints support pagination
3. **Limits**: Maximum query limits prevent resource exhaustion
4. **JSONB**: Efficient storage and querying of event data
5. **Async Logging**: Event emission doesn't block main operations

## Best Practices

1. **Regular Monitoring**: Check audit logs regularly for anomalies
2. **Retention Policy**: Implement log retention based on compliance requirements
3. **Alerting**: Set up alerts for suspicious patterns
4. **Access Review**: Regularly review who has access to audit logs
5. **Backup**: Include audit logs in backup strategy

## Testing

The system includes comprehensive test coverage:

- ✅ Audit log creation on document generation
- ✅ Admin can get audit logs with filters
- ✅ Admin can get statistics
- ✅ Admin can search logs
- ✅ User can get plan summary
- ✅ User can get own activity
- ✅ Authentication requirements
- ✅ Authorization checks

Run tests with:

```bash
cargo test --test will_audit_logs_test
```

## Related Documentation

- [Will Event Logging](./EVENTS.md)
- [Document Download API](./LEGAL_DOCUMENT_DOWNLOAD_API.md)
- [Authentication](./AUTHENTICATION.md)
- [Admin API](./ADMIN_API.md)
