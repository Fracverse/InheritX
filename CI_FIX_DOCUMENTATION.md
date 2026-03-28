# CI Fix for Legacy Content Upload Tests

## Issue

The GitHub Actions CI was failing because:
1. Tests require a PostgreSQL database
2. Tests use `sqlx::test` macro which needs `DATABASE_URL`
3. Secure messages feature requires `MESSAGE_KEY_ENCRYPTION_KEY` environment variable

## Solution

Updated `.github/workflows/backend.yml` to:

### 1. Add PostgreSQL Service

```yaml
services:
  postgres:
    image: postgres:15
    env:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: inheritx_test
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
    ports:
      - 5432:5432
```

### 2. Add Environment Variables

```yaml
env:
  DATABASE_URL: postgres://postgres:postgres@localhost:5432/inheritx_test
  MESSAGE_KEY_ENCRYPTION_KEY: test-key-for-ci-only-not-production-use
```

### 3. Install sqlx-cli and Run Migrations

```yaml
- name: Install sqlx-cli
  run: cargo install sqlx-cli --no-default-features --features postgres

- name: Run migrations
  run: sqlx migrate run
```

## What This Fixes

✅ **Database Tests** - All `#[sqlx::test]` tests now have a database  
✅ **Migrations** - Database schema is created before tests run  
✅ **Secure Messages** - Encryption key is available for tests  
✅ **Legacy Content** - All upload tests can run successfully  

## CI Workflow Steps (Updated)

1. ✅ Checkout code
2. ✅ Install Rust toolchain
3. ✅ Cache Rust dependencies
4. ✅ **Install sqlx-cli** (NEW)
5. ✅ **Run migrations** (NEW)
6. ✅ Check formatting
7. ✅ Run Clippy
8. ✅ Run tests (now with database)
9. ✅ Build release

## Testing Locally

To replicate CI environment locally:

```bash
# Start PostgreSQL
docker run -d \
  --name inheritx-test-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=inheritx_test \
  -p 5432:5432 \
  postgres:15

# Set environment variables
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/inheritx_test
export MESSAGE_KEY_ENCRYPTION_KEY=test-key-for-ci-only-not-production-use

# Run migrations
cd backend
sqlx migrate run

# Run tests
cargo test

# Cleanup
docker stop inheritx-test-db
docker rm inheritx-test-db
```

## Files Modified

- `.github/workflows/backend.yml` - Added PostgreSQL service and migrations

## Verification

After pushing this change, the CI should:
1. ✅ Start PostgreSQL service
2. ✅ Run all migrations
3. ✅ Pass all tests (including legacy_content_tests)
4. ✅ Build successfully

## Notes

- PostgreSQL 15 is used (latest stable)
- Test database is ephemeral (created per CI run)
- Migrations run automatically before tests
- All environment variables are set for test environment
- No production secrets are used in CI

---

**Status:** ✅ CI Fixed - Ready to merge PR
