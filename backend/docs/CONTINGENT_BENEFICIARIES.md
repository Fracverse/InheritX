# Contingent Beneficiaries System

## Overview

The Contingent Beneficiaries System provides backup beneficiary support for inheritance plans. When primary beneficiaries cannot claim (due to death, decline, or timeout), contingent beneficiaries are automatically activated to ensure assets always have a recipient.

## Key Features

### 1. Add Contingent Beneficiary (`add_contingent_beneficiary`)

**Purpose**: Add backup beneficiaries to inheritance plans

**Endpoint**: `POST /api/plans/:plan_id/beneficiaries/contingent`

**Request**:

```json
{
  "walletAddress": "0xCONTINGENT123...",
  "allocationPercent": "100.00",
  "name": "Jane Doe",
  "relationship": "Sister",
  "priorityOrder": 1
}
```

**Validation**:

- Plan must exist and belong to user
- Allocation percent must be between 0 and 100
- Priority order determines activation sequence
- Wallet address must be unique per plan

**Effects**:

- Creates contingent beneficiary record
- Sets beneficiary_type to 'contingent'
- Sets is_active to false (inactive until activated)
- Emits audit log event
- Sends notification to plan owner

**Example**:

```bash
curl -X POST http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/beneficiaries/contingent \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "walletAddress": "0xCONTINGENT123",
    "allocationPercent": "100.00",
    "name": "Jane Doe",
    "relationship": "Sister",
    "priorityOrder": 1
  }'
```

### 2. Remove Contingent Beneficiary (`remove_contingent_beneficiary`)

**Purpose**: Remove backup beneficiaries from plans

**Endpoint**: `DELETE /api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id`

**Validation**:

- Beneficiary must exist and belong to user's plan
- Can only remove contingent beneficiaries (not primary)
- User must own the plan

**Effects**:

- Deletes beneficiary record
- Cascades to related contingency conditions
- Emits audit log event

**Example**:

```bash
curl -X DELETE http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/beneficiaries/contingent/660e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <token>"
```

### 3. Get Contingent Beneficiaries (`get_contingent_beneficiaries`)

**Purpose**: List all contingent beneficiaries for a plan

**Endpoint**: `GET /api/plans/:plan_id/beneficiaries/contingent`

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "planId": "550e8400-e29b-41d4-a716-446655440000",
      "walletAddress": "0xCONTINGENT123",
      "allocationPercent": "100.00",
      "name": "Jane Doe",
      "relationship": "Sister",
      "beneficiaryType": "contingent",
      "priorityOrder": 1,
      "isActive": false,
      "activatedAt": null,
      "activationReason": null,
      "createdAt": "2026-04-24T10:00:00Z",
      "updatedAt": "2026-04-24T10:00:00Z"
    }
  ]
}
```

**Sorting**: Results ordered by priority_order ASC, then created_at ASC

**Example**:

```bash
curl -X GET http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/beneficiaries/contingent \
  -H "Authorization: Bearer <token>"
```

### 4. Set Contingency Conditions (`set_contingency_conditions`)

**Purpose**: Define when contingent beneficiaries should be activated

**Endpoint**: `POST /api/plans/:plan_id/contingency/conditions`

**Request**:

```json
{
  "primaryBeneficiaryId": "770e8400-e29b-41d4-a716-446655440002",
  "contingentBeneficiaryId": "660e8400-e29b-41d4-a716-446655440001",
  "conditionType": "primary_timeout",
  "timeoutDays": 30
}
```

**Condition Types**:

- `primary_declined`: Primary beneficiary explicitly declines
- `primary_deceased`: Primary beneficiary deceased (oracle verification)
- `primary_timeout`: Primary fails to claim within X days
- `manual_promotion`: Manual promotion by plan owner

**Validation**:

- Plan must belong to user
- Both beneficiaries must exist
- timeout_days required for primary_timeout condition
- Unique constraint on (plan_id, primary_id, contingent_id, condition_type)

**Effects**:

- Creates or updates contingency condition
- Sets is_active to true
- Emits audit log event

**Example**:

```bash
curl -X POST http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/contingency/conditions \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "primaryBeneficiaryId": "770e8400-e29b-41d4-a716-446655440002",
    "contingentBeneficiaryId": "660e8400-e29b-41d4-a716-446655440001",
    "conditionType": "primary_timeout",
    "timeoutDays": 30
  }'
```

### 5. Promote Contingent (`promote_contingent`)

**Purpose**: Manually promote contingent beneficiary to primary

**Endpoint**: `POST /api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id/promote`

**Request**:

```json
{
  "reason": "Primary beneficiary deceased"
}
```

**Validation**:

- Beneficiary must exist and be contingent type
- User must own the plan
- Reason must be provided

**Effects**:

- Changes beneficiary_type from 'contingent' to 'primary'
- Sets is_active to true
- Sets activated_at to current timestamp
- Records activation_reason
- Creates promotion history record
- Creates activation event record
- Emits audit log event
- Sends notification to plan owner

**Example**:

```bash
curl -X POST http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/beneficiaries/contingent/660e8400-e29b-41d4-a716-446655440001/promote \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Primary beneficiary deceased"
  }'
```

### 6. Get Contingency Config (`get_or_create_config`)

**Purpose**: Get or create contingency configuration for a plan

**Endpoint**: `GET /api/plans/:plan_id/contingency/config`

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "880e8400-e29b-41d4-a716-446655440003",
    "planId": "550e8400-e29b-41d4-a716-446655440000",
    "primaryClaimTimeoutDays": 30,
    "contingentClaimTimeoutDays": 30,
    "autoActivateOnTimeout": true,
    "requireManualConfirmation": false,
    "createdAt": "2026-04-24T10:00:00Z",
    "updatedAt": "2026-04-24T10:00:00Z"
  }
}
```

**Default Configuration**:

- Primary claim timeout: 30 days
- Contingent claim timeout: 30 days
- Auto-activate on timeout: true
- Require manual confirmation: false

**Example**:

```bash
curl -X GET http://localhost:3000/api/plans/550e8400-e29b-41d4-a716-446655440000/contingency/config \
  -H "Authorization: Bearer <token>"
```

## Activation Logic

### Automatic Activation (Timeout-Based)

The system includes a background job that checks for expired primary claim timeouts:

```rust
pub async fn check_and_activate_timeouts(pool: &PgPool) -> Result<Vec<Uuid>, ApiError>
```

**Process**:

1. Find plans where:
   - Status is 'active'
   - auto_activate_on_timeout is true
   - contract_created_at + primary_claim_timeout_days < NOW()
   - No claims exist for the plan
2. For each expired plan:
   - Activate all contingent beneficiaries
   - Set is_active = true
   - Record activation_reason = 'primary_timeout'
   - Create activation event records
3. Return list of activated plan IDs

**Scheduling**: Run every hour via cron job or background worker

### Manual Activation

Plan owners can manually activate contingent beneficiaries via:

- `promote_contingent()` - Promotes specific contingent to primary
- Useful when primary beneficiary explicitly declines or is deceased

### Condition-Based Activation

Contingency conditions define when activation should occur:

- **primary_declined**: Triggered when primary explicitly declines claim
- **primary_deceased**: Triggered by oracle verification of death
- **primary_timeout**: Triggered automatically after timeout period
- **manual_promotion**: Triggered by plan owner action

## Database Schema

### Extended plan_beneficiaries Table

```sql
ALTER TABLE plan_beneficiaries ADD COLUMN beneficiary_type beneficiary_type DEFAULT 'primary';
ALTER TABLE plan_beneficiaries ADD COLUMN priority_order INTEGER DEFAULT 0;
ALTER TABLE plan_beneficiaries ADD COLUMN is_active BOOLEAN DEFAULT true;
ALTER TABLE plan_beneficiaries ADD COLUMN activated_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE plan_beneficiaries ADD COLUMN activation_reason VARCHAR(100);
```

### contingency_conditions Table

```sql
CREATE TABLE contingency_conditions (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id),
    primary_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id),
    contingent_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id),
    condition_type contingency_condition NOT NULL,
    timeout_days INTEGER,
    is_active BOOLEAN DEFAULT true,
    activated_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(plan_id, primary_beneficiary_id, contingent_beneficiary_id, condition_type)
);
```

### contingent_activation_events Table

```sql
CREATE TABLE contingent_activation_events (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id),
    primary_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id),
    contingent_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id),
    activation_reason contingency_condition NOT NULL,
    activated_by_user_id UUID REFERENCES users(id),
    activated_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE
);
```

### contingent_promotions Table

```sql
CREATE TABLE contingent_promotions (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id),
    beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id),
    promoted_from_type beneficiary_type NOT NULL,
    promoted_to_type beneficiary_type NOT NULL,
    promotion_reason VARCHAR(255),
    promoted_by_user_id UUID REFERENCES users(id),
    promoted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE
);
```

### contingency_config Table

```sql
CREATE TABLE contingency_config (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id),
    primary_claim_timeout_days INTEGER DEFAULT 30,
    contingent_claim_timeout_days INTEGER DEFAULT 30,
    auto_activate_on_timeout BOOLEAN DEFAULT true,
    require_manual_confirmation BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(plan_id)
);
```

## Use Cases

### Use Case 1: Standard Estate Planning

**Scenario**: User wants sister as backup if spouse cannot claim

**Steps**:

1. Create plan with spouse as primary beneficiary
2. Add sister as contingent beneficiary (priority 1)
3. Set condition: primary_timeout with 30 days
4. If spouse doesn't claim within 30 days, sister is automatically activated

```bash
# Add contingent beneficiary
curl -X POST /api/plans/:plan_id/beneficiaries/contingent \
  -d '{"walletAddress": "0xSISTER", "allocationPercent": "100", "priorityOrder": 1}'

# Set timeout condition
curl -X POST /api/plans/:plan_id/contingency/conditions \
  -d '{"primaryBeneficiaryId": "...", "contingentBeneficiaryId": "...", "conditionType": "primary_timeout", "timeoutDays": 30}'
```

### Use Case 2: Multiple Contingent Beneficiaries

**Scenario**: User wants multiple backups in priority order

**Steps**:

1. Add contingent beneficiary 1 (priority 1) - Sister
2. Add contingent beneficiary 2 (priority 2) - Brother
3. Add contingent beneficiary 3 (priority 3) - Charity
4. If primary fails, activate in order: Sister → Brother → Charity

```bash
# Add multiple contingents with different priorities
curl -X POST /api/plans/:plan_id/beneficiaries/contingent \
  -d '{"walletAddress": "0xSISTER", "priorityOrder": 1, ...}'

curl -X POST /api/plans/:plan_id/beneficiaries/contingent \
  -d '{"walletAddress": "0xBROTHER", "priorityOrder": 2, ...}'

curl -X POST /api/plans/:plan_id/beneficiaries/contingent \
  -d '{"walletAddress": "0xCHARITY", "priorityOrder": 3, ...}'
```

### Use Case 3: Manual Promotion

**Scenario**: Primary beneficiary explicitly declines inheritance

**Steps**:

1. Primary beneficiary contacts plan owner
2. Plan owner manually promotes contingent to primary
3. System records reason and creates audit trail

```bash
curl -X POST /api/plans/:plan_id/beneficiaries/contingent/:id/promote \
  -d '{"reason": "Primary beneficiary declined inheritance"}'
```

### Use Case 4: Oracle-Based Activation

**Scenario**: Death oracle verifies primary beneficiary deceased

**Steps**:

1. Oracle service detects death certificate
2. Oracle calls activation API with proof
3. System activates contingent beneficiaries
4. Notification sent to contingent beneficiaries

## Error Handling

### Common Errors

| Error                                                             | Cause                           | Solution                        |
| ----------------------------------------------------------------- | ------------------------------- | ------------------------------- |
| `BadRequest: allocation_percent must be between 0 and 100`        | Invalid allocation              | Use value between 0-100         |
| `NotFound: Plan X not found`                                      | Plan doesn't exist or not owned | Verify plan ID and ownership    |
| `BadRequest: Can only remove contingent beneficiaries`            | Trying to remove primary        | Only contingent can be removed  |
| `BadRequest: Beneficiary is not contingent type`                  | Trying to promote primary       | Only contingent can be promoted |
| `BadRequest: timeout_days required for primary_timeout condition` | Missing timeout                 | Provide timeout_days            |
| `Forbidden: Not authorized`                                       | User doesn't own plan           | Verify ownership                |

## Security Considerations

### Authorization

- All operations require user authentication
- Users can only modify their own plans
- Beneficiary operations verify plan ownership

### Input Validation

- Allocation percent validated (0-100)
- Beneficiary type validated (primary/contingent)
- Condition type validated against enum
- Timeout days validated as positive integer

### Audit Trail

- All operations logged with user ID
- Promotion history tracked
- Activation events recorded
- Immutable audit logs

### Data Integrity

- Foreign key constraints ensure referential integrity
- Unique constraints prevent duplicates
- Cascading deletes maintain consistency
- Transaction safety with rollback on error

## Performance Considerations

### Indexes

- `idx_contingency_conditions_plan_id` - Fast plan lookups
- `idx_contingent_activation_events_plan_id` - Fast event queries
- `idx_contingent_promotions_plan_id` - Fast promotion history
- `idx_contingent_claim_attempts_status` - Fast status filtering

### Query Optimization

- Beneficiary queries use plan_id index
- Activation checks use composite indexes
- Timeout checks use contract_created_at index

### Background Jobs

- Timeout checker runs hourly
- Processes only active plans
- Batches activation operations
- Logs errors for manual review

## Testing

### Unit Tests

Located in `backend/tests/contingent_beneficiary_tests.rs`

Tests cover:

- Add contingent beneficiary validation
- Allocation percent validation
- Beneficiary type distinction
- Priority order sorting
- Activation state management
- Promotion logic
- Timeout calculations
- Sequential activation
- Removal restrictions

### Run Tests

```bash
cargo test contingent_beneficiary
```

## Future Enhancements

### Phase 2: Advanced Features

- **Multi-level contingency**: Support for tertiary beneficiaries
- **Conditional allocation**: Different allocations based on conditions
- **Time-based vesting**: Gradual release of inheritance
- **Geographic restrictions**: Beneficiary location requirements

### Phase 3: Oracle Integration

- **Death verification**: Integrate with death certificate oracles
- **Identity verification**: KYC for contingent beneficiaries
- **Automated notifications**: Email/SMS to contingent beneficiaries
- **Legal compliance**: Jurisdiction-specific rules

### Phase 4: Analytics

- **Activation metrics**: Track activation rates and reasons
- **Beneficiary analytics**: Common patterns and trends
- **Risk assessment**: Identify high-risk configurations
- **Reporting**: Generate compliance reports

## References

- [Beneficiary Sync Documentation](./BENEFICIARY_SYNC.md)
- [Plan Service Documentation](../src/service.rs)
- [Event System Documentation](./EVENTS.md)
- [Audit Logging Documentation](./AUDIT_LOGS.md)
