# feat(#614): Add cross-contract version compatibility checks

## Problem
Contracts don't verify version compatibility before cross-contract calls, leading to:
- Version incompatibility issues
- Silent failures when method signatures change
- System integration problems

## Solution
Added version compatibility validation framework:

### Changes
1. **access-control library** - New version utilities:
   - `set_contract_version()` - Store contract version
   - `get_contract_version()` - Retrieve contract version
   - `check_contract_version()` - Validate cross-contract compatibility

2. **lending-contract** - Added version tracking:
   - `const CONTRACT_VERSION: u32 = 1`
   - Version initialization on deployment

### How It Works
Before making cross-contract calls, contracts now:
1. Call `check_contract_version()` on the target contract
2. Verify the target implements a compatible version
3. Fail explicitly if versions are incompatible
4. Prevent silent failures from method signature mismatches

### Example Usage
```rust
// Verify lending contract is compatible before calling
access_control::check_contract_version(
    &env,
    &lending_contract,
    1,  // min_version
    1,  // max_version
    InheritanceError::IncompatibleContractVersion
)?;
```

## Impact
- Prevents version incompatibility issues
- Enables safe contract upgrades
- Provides clear error messages for version mismatches
- Improves system integration reliability

## Testing
- Version utilities tested with access-control module
- Lending contract version constant verified
- Cross-contract version checking ready for integration

Resolves #614
