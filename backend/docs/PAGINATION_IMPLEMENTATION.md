# Pagination Implementation Summary

## Overview
This document tracks the implementation of pagination across all list endpoints in the InheritX backend API.

## Pagination Standards

### Query Parameters
- `page`: Page number (1-indexed, default: 1)
- `limit`: Items per page (default: 20, max: 100)
- `cursor`: For cursor-based pagination (optional)

### Response Format
```json
{
  "status": "success",
  "data": [...],
  "page": 1,
  "limit": 20,
  "total_count": 100,
  "total_pages": 5,
  "has_next": true,
  "has_prev": false
}
```

### Cursor-Based Pagination Response
```json
{
  "status": "success",
  "data": [...],
  "limit": 20,
  "total_count": 100,
  "has_next": true,
  "next_cursor": "base64encodedcursor"
}
```

## Implementation Status

### ✅ Completed Endpoints

1. **GET /api/notifications** - User notifications
   - Added `PaginationQuery` parameter
   - Uses `list_for_user_paginated` and `count_for_user`
   - Returns paginated response with metadata

2. **GET /api/admin/logs** - Admin audit logs
   - Added `PaginationQuery` parameter
   - Uses `list_all_paginated` and `count_all`
   - Returns paginated response with metadata

3. **GET /api/plans/due-for-claim** - User due plans
   - Added `PaginationQuery` parameter
   - Uses `get_all_due_for_claim_plans_for_user_paginated` and `count_due_for_claim_plans_for_user`
   - Returns paginated response with metadata

4. **GET /api/admin/plans/due-for-claim** - Admin due plans
   - Added `PaginationQuery` parameter
   - Uses `get_all_due_for_claim_plans_admin_paginated` and `count_due_for_claim_plans_admin`
   - Returns paginated response with metadata

5. **GET /api/emergency/contacts** - Emergency contacts
   - Added `PaginationQuery` parameter
   - Uses `list_for_user_paginated` and `count_for_user`
   - Returns paginated response with metadata

6. **GET /api/plans/:plan_id/will/versions** - Will versions (already implemented)
   - Uses custom `PaginationParams`
   - Returns `PaginatedVersions` response

### 🔄 In Progress / Pending Endpoints

7. **GET /api/messages/legacy** - Legacy messages
   - Service: `MessageEncryptionService::list_owner_messages`
   - Needs: Pagination parameters and count method

8. **GET /api/messages/legacy/vault/:vault_id** - Vault messages
   - Service: `MessageEncryptionService::list_vault_messages`
   - Needs: Pagination parameters and count method

9. **GET /api/admin/messages/keys** - Message encryption keys
   - Service: `MessageKeyService::list_keys`
   - Needs: Pagination parameters and count method

10. **GET /api/admin/messages/audit** - Message audit logs
    - Service: `MessageAccessAuditService::get_logs`
    - Already has filters with limit/offset
    - Needs: Standardized pagination response

11. **GET /api/emergency/access/audit-logs** - Emergency access audit logs
    - Service: `EmergencyAccessService::list_audit_logs`
    - Needs: Pagination parameters and count method

12. **GET /api/emergency/access/risk-alerts** - Risk alerts
    - Service: `EmergencyAccessService::list_risk_alerts`
    - Needs: Pagination parameters and count method

13. **GET /api/emergency/access/sessions** - Active emergency sessions
    - Service: `EmergencySessionService::list_active_sessions`
    - Needs: Pagination parameters and count method

14. **GET /api/loans/lifecycle** - Lifecycle loans
    - Service: `LoanLifecycleService::list_loans`
    - Already has filters
    - Needs: Pagination parameters and count method

15. **GET /api/admin/emergency-access/all** - All emergency access records
    - Service: `LegacyEmergencyAccessService::get_all_access`
    - Needs: Pagination parameters and count method

16. **GET /api/admin/emergency-access/active-sessions** - Active emergency sessions (admin)
    - Service: `LegacyEmergencyAccessService::get_active_sessions`
    - Needs: Pagination parameters and count method

17. **GET /api/admin/emergency/paused-plans** - Paused plans
    - Service: `EmergencyAdminService::get_paused_plans`
    - Needs: Pagination parameters and count method

18. **GET /api/admin/emergency/risk-override-plans** - Risk override plans
    - Service: `EmergencyAdminService::get_risk_override_plans`
    - Needs: Pagination parameters and count method

19. **GET /api/governance/proposals** - Governance proposals
    - Service: `GovernanceService::list_proposals`
    - Needs: Pagination parameters and count method

20. **GET /api/plans/:plan_id/will/documents** - Will documents
    - Service: `WillPdfService::list_for_plan`
    - Needs: Pagination parameters and count method

21. **GET /api/will/documents/:document_id/backups** - Document backups
    - Service: `DocumentStorageService::list_backups`
    - Needs: Pagination parameters and count method

22. **GET /api/will/documents/:document_id/witnesses** - Witnesses
    - Service: `WitnessService::get_witnesses`
    - Needs: Pagination parameters and count method

23. **GET /api/will/documents/:document_id/signatures** - Will signatures
    - Service: `WillSignatureService::get_signatures_for_document`
    - Needs: Pagination parameters and count method

24. **GET /api/will/documents/:document_id/events** - Document events
    - Service: `WillEventService::get_document_events`
    - Needs: Pagination parameters and count method

25. **GET /api/plans/:plan_id/will/events** - Plan events
    - Service: `WillEventService::get_plan_events`
    - Needs: Pagination parameters and count method

26. **GET /api/will/vaults/:vault_id/events** - Vault events
    - Service: `WillEventService::get_vault_events`
    - Needs: Pagination parameters and count method

27. **GET /api/admin/will/audit/logs** - Admin audit logs
    - Service: `WillAuditService::get_audit_logs`
    - Already has `AuditLogFilters` with limit/offset
    - Needs: Standardized pagination response

28. **GET /api/admin/insurance-funds** - All insurance funds
    - Service: `InsuranceFundService::get_all_funds`
    - Needs: Pagination parameters and count method

29. **GET /api/admin/insurance-fund/:fund_id/metrics** - Insurance fund metrics history
    - Already has days parameter
    - Needs: Pagination parameters and count method

30. **GET /api/admin/insurance-fund/:fund_id/transactions** - Insurance fund transactions
    - Already has limit parameter
    - Needs: Full pagination with offset and count

31. **GET /api/admin/insurance-fund/:fund_id/claims** - Insurance claims
    - Already has limit and status filter
    - Needs: Full pagination with offset and count

32. **GET /api/content** - User legacy content
    - Service: `LegacyContentService::list_user_content`
    - Already has `ContentListFilters` with limit/offset
    - Needs: Standardized pagination response

33. **GET /api/plans/:plan_id/beneficiaries/contingent** - Contingent beneficiaries
    - Service: `ContingentBeneficiaryService::get_contingent_beneficiaries`
    - Needs: Pagination parameters and count method

## Testing Requirements

### Unit Tests
- ✅ Pagination module tests (encode/decode cursor, normalize parameters)
- ⏳ Service-level pagination tests for each endpoint
- ⏳ Count method tests

### Integration Tests
- ⏳ Update `backend/tests/pagination_tests.rs` (currently commented out)
- ⏳ Test pagination with various page sizes
- ⏳ Test edge cases (empty results, single page, last page)
- ⏳ Test cursor-based pagination
- ⏳ Test pagination metadata accuracy

## Performance Considerations

### Optimizations Implemented
1. **Limit clamping**: Max 100 items per page to prevent large queries
2. **Index usage**: Ensure all paginated queries use appropriate indexes
3. **Count optimization**: Consider caching counts for expensive queries

### Future Optimizations
1. **Cursor-based pagination**: For large datasets, implement cursor-based pagination
2. **Count caching**: Cache total counts with TTL for frequently accessed endpoints
3. **Materialized views**: For complex aggregations in due plans queries

## Migration Notes

### Breaking Changes
- All list endpoints now return pagination metadata
- Clients must handle the new response format
- Default limit is 20 (previously unlimited)

### Backward Compatibility
- Endpoints work without pagination parameters (use defaults)
- Existing clients will receive paginated results with default values

## API Documentation Updates

All endpoint documentation should include:
- Query parameters: `page`, `limit`, `cursor` (where applicable)
- Response format with pagination metadata
- Examples with pagination
- Performance notes for large datasets

## Next Steps

1. ✅ Create pagination module
2. ✅ Update core endpoints (notifications, admin logs, due plans, emergency contacts)
3. ⏳ Update remaining list endpoints (messages, loans, governance, etc.)
4. ⏳ Add count methods to all services
5. ⏳ Implement cursor-based pagination for high-volume endpoints
6. ⏳ Update and uncomment pagination tests
7. ⏳ Update API documentation
8. ⏳ Performance testing with large datasets
9. ⏳ Add database indexes where needed

## Database Indexes

Recommended indexes for pagination performance:

```sql
-- Notifications
CREATE INDEX IF NOT EXISTS idx_notifications_user_created 
ON notifications(user_id, created_at DESC);

-- Action logs
CREATE INDEX IF NOT EXISTS idx_action_logs_timestamp 
ON action_logs(timestamp DESC);

-- Emergency contacts
CREATE INDEX IF NOT EXISTS idx_emergency_contacts_user_created 
ON emergency_contacts(user_id, created_at DESC);

-- Plans
CREATE INDEX IF NOT EXISTS idx_plans_user_status_created 
ON plans(user_id, status, created_at DESC);

-- Add similar indexes for other paginated tables
```
