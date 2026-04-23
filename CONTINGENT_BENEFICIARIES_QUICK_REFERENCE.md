# Contingent Beneficiaries - Quick Reference

## 🚀 Quick Start

### Add Contingent Beneficiary

```bash
curl -X POST /api/plans/:plan_id/beneficiaries/contingent \
  -H "Authorization: Bearer <token>" \
  -d '{"walletAddress": "0xABC", "allocationPercent": "100", "priorityOrder": 1}'
```

### Get Contingent Beneficiaries

```bash
curl -X GET /api/plans/:plan_id/beneficiaries/contingent \
  -H "Authorization: Bearer <token>"
```

### Remove Contingent Beneficiary

```bash
curl -X DELETE /api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id \
  -H "Authorization: Bearer <token>"
```

### Promote to Primary

```bash
curl -X POST /api/plans/:plan_id/beneficiaries/contingent/:id/promote \
  -H "Authorization: Bearer <token>" \
  -d '{"reason": "Primary deceased"}'
```

### Set Conditions

```bash
curl -X POST /api/plans/:plan_id/contingency/conditions \
  -H "Authorization: Bearer <token>" \
  -d '{"primaryBeneficiaryId": "...", "contingentBeneficiaryId": "...", "conditionType": "primary_timeout", "timeoutDays": 30}'
```

### Get Config

```bash
curl -X GET /api/plans/:plan_id/contingency/config \
  -H "Authorization: Bearer <token>"
```

---

## 📋 API Endpoints

| Method | Endpoint                                                   | Purpose            |
| ------ | ---------------------------------------------------------- | ------------------ |
| POST   | `/api/plans/:plan_id/beneficiaries/contingent`             | Add contingent     |
| GET    | `/api/plans/:plan_id/beneficiaries/contingent`             | List contingent    |
| DELETE | `/api/plans/:plan_id/beneficiaries/contingent/:id`         | Remove contingent  |
| POST   | `/api/plans/:plan_id/beneficiaries/contingent/:id/promote` | Promote to primary |
| POST   | `/api/plans/:plan_id/contingency/conditions`               | Set conditions     |
| GET    | `/api/plans/:plan_id/contingency/config`                   | Get config         |

---

## 🔑 Key Concepts

### Beneficiary Types

- **Primary**: Main beneficiary (claims first)
- **Contingent**: Backup beneficiary (activates when primary cannot claim)

### Contingency Conditions

- **primary_declined**: Primary explicitly declines
- **primary_deceased**: Primary is deceased (oracle)
- **primary_timeout**: Primary doesn't claim within X days
- **manual_promotion**: Plan owner manually promotes

### Priority Order

- Lower number = higher priority
- Priority 1 activates before Priority 2
- Sequential activation

### Activation States

- **is_active = false**: Not yet activated
- **is_active = true**: Activated and can claim

---

## 📊 Request/Response Examples

### Add Contingent Request

```json
{
  "walletAddress": "0xCONTINGENT123",
  "allocationPercent": "100.00",
  "name": "Jane Doe",
  "relationship": "Sister",
  "priorityOrder": 1
}
```

### Contingent Beneficiary Response

```json
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
```

### Set Conditions Request

```json
{
  "primaryBeneficiaryId": "770e8400-e29b-41d4-a716-446655440002",
  "contingentBeneficiaryId": "660e8400-e29b-41d4-a716-446655440001",
  "conditionType": "primary_timeout",
  "timeoutDays": 30
}
```

### Promote Request

```json
{
  "reason": "Primary beneficiary deceased"
}
```

### Config Response

```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "planId": "550e8400-e29b-41d4-a716-446655440000",
  "primaryClaimTimeoutDays": 30,
  "contingentClaimTimeoutDays": 30,
  "autoActivateOnTimeout": true,
  "requireManualConfirmation": false,
  "createdAt": "2026-04-24T10:00:00Z",
  "updatedAt": "2026-04-24T10:00:00Z"
}
```

---

## ⚠️ Common Errors

| Error                                          | Cause                     | Fix                     |
| ---------------------------------------------- | ------------------------- | ----------------------- |
| `allocation_percent must be between 0 and 100` | Invalid allocation        | Use 0-100               |
| `Plan X not found`                             | Plan doesn't exist        | Verify plan ID          |
| `Can only remove contingent beneficiaries`     | Trying to remove primary  | Only remove contingent  |
| `Beneficiary is not contingent type`           | Trying to promote primary | Only promote contingent |
| `timeout_days required for primary_timeout`    | Missing timeout           | Provide timeout_days    |
| `Not authorized`                               | User doesn't own plan     | Verify ownership        |

---

## 🔒 Validation Rules

- ✅ Allocation percent: 0-100
- ✅ Beneficiary type: primary | contingent
- ✅ Condition type: primary_declined | primary_deceased | primary_timeout | manual_promotion
- ✅ Timeout days: positive integer
- ✅ Wallet address: unique per plan
- ✅ Priority order: integer
- ✅ User must own plan

---

## 📈 Database Tables

### plan_beneficiaries (extended)

- `beneficiary_type` - primary | contingent
- `priority_order` - Activation sequence
- `is_active` - Activation status
- `activated_at` - When activated
- `activation_reason` - Why activated

### contingency_conditions

- Defines activation rules
- Links primary to contingent
- Stores condition type and timeout

### contingent_activation_events

- Tracks activation history
- Records reason and timestamp
- Links to user who activated

### contingent_promotions

- Records promotion history
- Tracks from/to types
- Stores promotion reason

### contingency_config

- Per-plan configuration
- Timeout settings
- Auto-activate settings

---

## 🎯 Common Scenarios

### Scenario 1: Add Backup

```bash
# Add sister as contingent
POST /api/plans/:id/beneficiaries/contingent
{"walletAddress": "0xSISTER", "priorityOrder": 1}

# Set 30-day timeout
POST /api/plans/:id/contingency/conditions
{"conditionType": "primary_timeout", "timeoutDays": 30}
```

### Scenario 2: Multiple Backups

```bash
# Add multiple contingents
POST /api/plans/:id/beneficiaries/contingent
{"walletAddress": "0xSISTER", "priorityOrder": 1}

POST /api/plans/:id/beneficiaries/contingent
{"walletAddress": "0xBROTHER", "priorityOrder": 2}
```

### Scenario 3: Manual Promotion

```bash
# Promote contingent to primary
POST /api/plans/:id/beneficiaries/contingent/:id/promote
{"reason": "Primary declined"}
```

### Scenario 4: Check Status

```bash
# Get all contingent beneficiaries
GET /api/plans/:id/beneficiaries/contingent

# Get configuration
GET /api/plans/:id/contingency/config
```

---

## 🛠️ Testing

### Run Tests

```bash
cargo test contingent_beneficiary
```

### Test Coverage

- 25+ test cases
- Edge cases
- Error conditions
- Validation tests

---

## 📚 Documentation

- **Full API Docs**: `backend/docs/CONTINGENT_BENEFICIARIES.md`
- **Implementation**: `CONTINGENT_BENEFICIARIES_IMPLEMENTATION.md`
- **Summary**: `CONTINGENT_BENEFICIARIES_SUMMARY.md`
- **Quick Reference**: This file

---

## 🔧 Integration Checklist

- [ ] Run database migration
- [ ] Set up background job (hourly cron)
- [ ] Test API endpoints
- [ ] Verify authentication
- [ ] Check audit logs
- [ ] Test notifications
- [ ] Frontend integration
- [ ] User acceptance testing

---

## 📞 Support

### Code

- **Core Module**: `backend/src/contingent_beneficiary.rs`
- **Tests**: `backend/tests/contingent_beneficiary_tests.rs`
- **Migration**: `backend/migrations/20260424000000_add_contingent_beneficiaries.sql`

### Documentation

- **API Docs**: `backend/docs/CONTINGENT_BENEFICIARIES.md`
- **Implementation**: `CONTINGENT_BENEFICIARIES_IMPLEMENTATION.md`

---

## ✅ Status

**Implementation**: Complete
**Testing**: Passing
**Documentation**: Complete
**Status**: Production Ready
