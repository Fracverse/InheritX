## Description
Add integration tests for the `/health` endpoint and implement a maturity check for plan claims. This ensures that plans can only be claimed after their intended distribution period (e.g., Monthly, Yearly) has elapsed.

Closes #101
Closes #116

## Changes proposed

### What were you told to do?
- Add integration test for `/health` endpoint to verify status and message.
- Implement maturity check for plan claims: claims should fail if the current date is before the plan's due date.
- Ensure all tests use the actual Axum router via `create_app`.
- Fix rate-limiting failures in integration tests.

### What did I do?

#### Backend Enhancements (`backend/src/service.rs`)
- Updated `PlanService::claim_plan`:
  - Added a check using `is_due_for_claim()` before allowing a claim to proceed.
  - Returns a `400 Bad Request` if the plan has not yet matured.

#### Server Configuration (`backend/src/main.rs`)
- Updated `axum::serve` to use `into_make_service_with_connect_info::<SocketAddr>()`.
- This ensures that the `GovernorLayer` (rate-limiting) can correctly identify client IP addresses, preventing 500 errors during high-frequency requests or integration tests.

#### Integration Tests (`backend/tests/`)
- **`health_tests.rs`**:
  - Verifies `GET /health` returns `200 OK` with the expected JSON payload.
  - Verifies `GET /health/db` for database connectivity.
- **`claim_tests.rs`**:
  - Tests the full claim flow from user creation and KYC approval to plan creation and claim attempts.
  - Asserts that immature plans (Monthly distribution, created < 30 days ago) return `400 Bad Request`.
  - Asserts that mature plans successfully return `200 OK`.

#### Testing Infrastructure
- All tests now spawn a real server on a random port using `tokio::net::TcpListener` and `axum::serve` for maximum fidelity.
- Uses `helpers::TestContext` for consistent test environment setup.

## Check List
- [x] Implements claim maturity validation logic in the backend service.
- [x] Includes comprehensive integration tests for health and claim endpoints.
- [x] Fixes rate-limiting middleware by providing necessary connection info.
- [x] Tests follow the project's requirement of using the actual Axum router.
