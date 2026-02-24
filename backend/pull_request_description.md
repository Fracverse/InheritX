# Pull Request: Add integration test for Web3 wallet login

## Description
This PR implements the Web3 wallet login feature (Stellar) and adds a comprehensive integration test suite as requested in issue #104.

### Key Changes
- **Backend Architecture**: Added a `nonces` table to support secure Web3 authentication flow.
- **Authentication Handlers**:
    - `get_nonce`: Generates and stores a unique nonce for a given wallet address.
    - `web3_login`: Verifies the Stellar wallet signature, finds/creates a user, and issues a JWT.
- **Security**: 
    - Implemented signature verification using the `ring` crate and `stellar-strkey` for address decoding.
    - Nonce invalidation after successful login to prevent replay attacks.
- **Integration Test**: Created `backend/tests/auth_tests.rs` which covers the full authentication flow using a real Axum router and a spawned server.

## Verification Results

### Automated Tests
All backend tests passed successfully, including the new `test_web3_login_success`.

```text
running 1 test
test test_web3_login_success ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Code Quality
- `cargo clippy`: Clean
- `cargo fmt`: Verified
- `cargo test`: All 7 tests passed (Health, Notifications, Web3 Auth)

## Proof of Work
Refer to the `walkthrough.md` for detailed exploration of the changes and test execution.
