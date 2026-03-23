# Emergency Access Activation - Soroban Contract

Fixes #259

## Summary

Implements emergency access activation functionality in the Soroban inheritance contract, allowing vault owners to designate a trusted contact who can access the vault in emergency situations.

## Changes

### Data Structures
- **EmergencyAccessRecord**: Tracks emergency access activation with plan_id, trusted_contact address, and activation timestamp
- **EmergencyAccessActivatedEvent**: Event emitted when emergency access is activated

### Storage
- Added `EmergencyAccess(u64)` DataKey variant for per-plan emergency access records

### Error Handling
- Added `EmergencyAccessAlreadyActive` error variant to prevent duplicate activations

### Functions

#### `activate_emergency_access()`
- **Access Control**: Owner-only (requires owner authorization)
- **Parameters**: 
  - `owner`: Plan owner address
  - `plan_id`: ID of the plan
  - `trusted_contact`: Address of the trusted contact
- **Behavior**:
  - Verifies caller is plan owner
  - Checks plan exists
  - Prevents duplicate activation
  - Records activation timestamp via `env.ledger().timestamp()`
  - Stores EmergencyAccessRecord in persistent storage
  - Emits EmergencyAccessActivatedEvent with all required data
  - Logs operation for audit trail
- **Errors**:
  - `Unauthorized`: If caller is not the plan owner
  - `PlanNotFound`: If plan doesn't exist
  - `EmergencyAccessAlreadyActive`: If already activated

#### `get_emergency_access()`
- **Access**: Public query function
- **Parameters**: `plan_id`
- **Returns**: EmergencyAccessRecord if active, None otherwise
- **Use Case**: Query current emergency access status

## Acceptance Criteria Met

✅ **Only owner can activate** - Function requires owner authorization via `owner.require_auth()`

✅ **Activation timestamp recorded** - Uses `env.ledger().timestamp()` to record exact activation time

✅ **Event emitted** - EmergencyAccessActivatedEvent published with:
- `plan_id`: The vault/plan ID
- `trusted_contact`: The designated emergency contact address
- `activated_at`: The activation timestamp

## Testing

- All 75 existing tests pass
- Code compiles without errors
- Follows existing contract patterns and conventions
- Consistent with other owner-only operations (add_beneficiary, remove_beneficiary, deactivate_inheritance_plan)

## Implementation Notes

- Uses two-part event topic: `(symbol_short!("EMERG"), symbol_short!("ACTIV"))`
- Follows existing storage pattern with DataKey enum
- Consistent error handling with other contract functions
- Includes comprehensive logging for audit trail
- Prevents accidental duplicate activations
