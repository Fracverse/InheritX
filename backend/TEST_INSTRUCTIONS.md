# Testing Instructions for Pagination Implementation

## Prerequisites

1. **Install Rust and Cargo** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Set up environment variables**:
   ```bash
   cp backend/env.example backend/.env
   # Edit .env with your database credentials
   ```

3. **Run database migrations**:
   ```bash
   cd backend
   sqlx migrate run
   ```

## Running Tests

### Run All Tests:
```bash
cd backend
cargo test
```

### Run Only Pagination Tests:
```bash
cd backend
cargo test pagination_tests
```

### Run with Detailed Output:
```bash
cd backend
cargo test pagination_tests -- --nocapture
```

### Check for Compilation Errors:
```bash
cd backend
cargo check
```

### Build the Project:
```bash
cd backend
cargo build
```

## Expected Test Results

The following tests should pass:

1. ✅ `notifications_endpoint_supports_page_and_limit` - Tests pagination on notifications endpoint
2. ✅ `admin_logs_endpoint_supports_page_and_limit` - Tests pagination on admin logs endpoint
3. ✅ `due_plans_endpoint_supports_page_and_limit` - Tests pagination on due plans endpoint
4. ✅ `create_plan_accepts_query_pagination_params` - Tests backward compatibility

## Potential Issues to Check

### 1. Database Connection
If tests fail with database errors:
- Ensure PostgreSQL is running
- Check DATABASE_URL in .env
- Verify migrations have been run

### 2. JWT Secret
If tests fail with authentication errors:
- Ensure JWT_SECRET is set in environment
- Default test secret is "test-jwt-secret"

### 3. Compilation Errors
If cargo check fails:
- Check that all imports are correct
- Verify base64 encoding is using the correct API (base64 v0.21)
- Ensure all service methods are properly defined

## Manual API Testing

Once the server is running:

```bash
# Start the server
cargo run

# Test notifications endpoint
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "http://localhost:8000/api/notifications?page=1&limit=10"

# Test admin logs endpoint
curl -H "Authorization: Bearer ADMIN_TOKEN" \
  "http://localhost:8000/api/admin/logs?page=1&limit=20"

# Test due plans endpoint
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "http://localhost:8000/api/plans/due-for-claim?page=1&limit=10"

# Test emergency contacts endpoint
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "http://localhost:8000/api/emergency/contacts?page=1&limit=10"

# Test loans endpoint with filters
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "http://localhost:8000/api/loans/lifecycle?status=active&page=1&limit=20"
```

## Expected Response Format

All paginated endpoints should return:

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

## Troubleshooting

### If tests fail:
1. Check the error message carefully
2. Verify database schema is up to date
3. Ensure all dependencies are installed: `cargo build`
4. Check that test data is being created correctly
5. Verify authentication tokens are valid

### Common Issues:
- **"column does not exist"**: Run migrations
- **"connection refused"**: Start PostgreSQL
- **"authentication failed"**: Check JWT_SECRET
- **"type mismatch"**: Check Rust version compatibility

## Success Criteria

All tests pass when:
- ✅ Pagination parameters are correctly parsed
- ✅ Data is properly limited and offset
- ✅ Pagination metadata is accurate
- ✅ Total counts match database records
- ✅ has_next and has_prev flags are correct
- ✅ Endpoints work without pagination params (defaults)

## Next Steps After Tests Pass

1. Update remaining endpoints with pagination
2. Add database indexes for performance
3. Update API documentation
4. Integrate with frontend
5. Performance test with large datasets
6. Deploy to staging environment
