# Backend Integration Tests

This directory contains integration tests for the InheritX backend, including security tests for JWT validation.

## Test Files

### `security_tests.rs` - JWT Security Tests

**Purpose**: Verify JWT signature validation prevents privilege escalation attacks

**Test Count**: 8 comprehensive tests

**Coverage**:
- ✅ Modified JWT payload rejection
- ✅ Valid token acceptance
- ✅ Missing header rejection
- ✅ Invalid format rejection
- ✅ Expired token rejection
- ✅ Malformed JWT rejection
- ✅ Algorithm mismatch rejection
- ✅ Empty token rejection

**Documentation**: See `SECURITY_TESTS.md`

### `health_tests.rs` - Health Check Tests

**Purpose**: Verify health check endpoints work correctly

**Tests**:
- Health check returns 200
- Database health check returns 200 when connected
- Database health check returns 500 when disconnected

## Running Tests

### Prerequisites

```bash
# Set environment variables
export DATABASE_URL="postgresql://user:password@localhost/inheritx_test"
export JWT_SECRET="your-test-jwt-secret"
```

### Run All Tests

```bash
cd ../
cargo test
```

### Run Only Security Tests

```bash
cd ../
cargo test --test security_tests -- --nocapture
```

### Run Specific Test

```bash
cd ../
cargo test --test security_tests test_modified_jwt_signature_rejected_on_admin_route -- --nocapture
```

### Run with Output

```bash
cd ../
cargo test --test security_tests -- --nocapture --test-threads=1
```

## Test Structure

### Integration Test Pattern

All tests follow this pattern:

```rust
#[tokio::test]
async fn test_name() {
    // 1. Setup: Load test context with database and app
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 2. Prepare: Create test data (JWT token, request, etc.)
    let token = create_token();
    let request = build_request(token);

    // 3. Execute: Make request through actual router
    let response = test_context.app.oneshot(request).await?;

    // 4. Assert: Verify response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### TestContext Helper

Located in `helpers/mod.rs`:

```rust
pub struct TestContext {
    pub app: Router,
    pub pool: PgPool,
}

impl TestContext {
    pub async fn from_env() -> Option<Self> {
        // Loads DATABASE_URL and JWT_SECRET from environment
        // Creates database pool
        // Initializes Axum router
        // Returns test context or None if setup fails
    }
}
```

## Security Test Details

### Attack Scenario

```
1. Attacker obtains valid user token
2. Decodes JWT payload
3. Modifies role: "user" → "admin"
4. Re-encodes with wrong secret
5. Attempts to access admin endpoint
```

### Expected Defense

```
Backend rejects with HTTP 401 Unauthorized
Signature verification fails
Privilege escalation prevented
```

### Test Implementation

```rust
#[tokio::test]
async fn test_modified_jwt_signature_rejected_on_admin_route() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // Create valid token
    let valid_token = encode(&Header::default(), &valid_claims, &key)?;

    // Modify payload (attacker action)
    let modified_claims = TestClaims {
        role: "admin",  // ← Escalation attempt
        ..
    };

    // Re-encode with wrong secret (attacker action)
    let tampered_token = encode(&Header::default(), &modified_claims, &wrong_key)?;

    // Attempt access
    let response = test_context.app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/admin/logs")
            .header("Authorization", format!("Bearer {}", tampered_token))
            .body(Body::empty())?
    ).await?;

    // Verify rejection
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

## Environment Variables

### Required

- `DATABASE_URL`: PostgreSQL connection string
  - Example: `postgresql://user:password@localhost/inheritx_test`
  - Required for all integration tests

- `JWT_SECRET`: Secret key for JWT signing
  - Example: `your-test-jwt-secret`
  - Must match backend configuration
  - Used by tests to create valid tokens

### Optional

- `RUST_LOG`: Logging level
  - Example: `RUST_LOG=debug cargo test`
  - Useful for debugging test failures

## Troubleshooting

### Tests Skip (DATABASE_URL not set)

```
Skipping integration test: DATABASE_URL is not set
```

**Solution**: Set DATABASE_URL environment variable

```bash
export DATABASE_URL="postgresql://localhost/inheritx_test"
```

### Connection Refused

```
unable to connect to DATABASE_URL: connection refused
```

**Solution**: Ensure PostgreSQL is running

```bash
# Check if PostgreSQL is running
psql -U postgres -c "SELECT 1"

# Or start PostgreSQL
brew services start postgresql  # macOS
sudo systemctl start postgresql # Linux
```

### JWT Secret Mismatch

Tests fail with 401 on valid tokens

**Solution**: Ensure JWT_SECRET matches backend config

```bash
export JWT_SECRET="your-backend-secret"
```

### Database Connection Pool Exhausted

```
connection pool exhausted
```

**Solution**: Reduce test parallelism

```bash
cargo test --test security_tests -- --test-threads=1
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run Integration Tests
  env:
    DATABASE_URL: ${{ secrets.TEST_DATABASE_URL }}
    JWT_SECRET: ${{ secrets.TEST_JWT_SECRET }}
  run: |
    cd backend
    cargo test --test security_tests -- --nocapture
```

### Pre-Commit Hook

```bash
#!/bin/bash
cd backend
cargo test --test security_tests -- --nocapture
if [ $? -ne 0 ]; then
    echo "Security tests failed!"
    exit 1
fi
```

## Performance

### Execution Time

- Each test: ~100-500ms (includes database operations)
- Full suite: ~1-2 seconds
- Acceptable for CI/CD

### Optimization

If tests become slow:

1. Use in-memory database (SQLite)
2. Mock database layer
3. Run tests in parallel
4. Pre-populate test fixtures

## Best Practices

### Writing New Tests

1. **Use TestContext**: Always load test context first
2. **Simulate Reality**: Use actual HTTP requests, not mocks
3. **Test One Thing**: Each test should verify one behavior
4. **Clear Names**: Test names should describe what they test
5. **Document**: Add comments explaining attack scenarios

### Test Naming Convention

```rust
// Pattern: test_<what>_<expected_result>
test_modified_jwt_signature_rejected_on_admin_route()
test_valid_jwt_signature_accepted_on_admin_route()
test_missing_authorization_header_rejected()
```

### Assertion Messages

```rust
// Good: Explains what failed and why
assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Modified JWT should be rejected with 401 Unauthorized"
);

// Bad: No context
assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
```

## Documentation

- `SECURITY_TESTS.md` - Detailed security test documentation
- `../SECURITY_TEST_GUIDE.md` - Quick reference guide
- `../IMPLEMENTATION_NOTES.md` - Technical deep-dive
- `../SECURITY_TEST_SUMMARY.md` - Implementation summary

## Related Files

- `../src/auth.rs` - JWT validation logic
- `../src/app.rs` - Route definitions
- `../Cargo.toml` - Dependencies
- `helpers/mod.rs` - Test utilities

## Questions?

1. **How do I run a specific test?**
   ```bash
   cargo test --test security_tests test_name -- --nocapture
   ```

2. **How do I debug a failing test?**
   ```bash
   RUST_LOG=debug cargo test --test security_tests -- --nocapture --test-threads=1
   ```

3. **How do I add a new test?**
   - Add function to `security_tests.rs`
   - Use `#[tokio::test]` attribute
   - Follow existing test pattern
   - Add documentation comment

4. **What if tests pass locally but fail in CI?**
   - Check environment variables in CI
   - Verify database is accessible
   - Check JWT_SECRET matches
   - Review CI logs for errors

## Security Checklist

Before deploying to production:

- [ ] All tests pass locally
- [ ] All tests pass in CI/CD
- [ ] JWT_SECRET is strong (32+ chars)
- [ ] JWT_SECRET is NOT in version control
- [ ] HTTPS is enforced
- [ ] Token expiration is reasonable
- [ ] No hardcoded secrets in code
- [ ] Rate limiting is enabled
- [ ] Audit logging is enabled

---

**Last Updated**: 2026-02-22  
**Status**: ✅ Production Ready  
**Test Count**: 8 security tests + 2 health tests
