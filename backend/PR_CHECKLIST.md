# PR Checklist - Pagination Implementation

## ⚠️ CRITICAL: You MUST test before creating a PR!

Without Cargo installed, you cannot verify if your code works. **This will cause your PR to fail CI/CD checks.**

## Before Creating the PR:

### Step 1: Install Rust & Cargo
```bash
# On Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# On Windows
# Download and run: https://rustup.rs/
```

### Step 2: Verify Installation
```bash
cargo --version
rustc --version
```

### Step 3: Set Up Environment
```bash
cd backend
cp env.example .env
# Edit .env with your database credentials
```

### Step 4: Run Database Migrations
```bash
cd backend
sqlx migrate run
```

### Step 5: Check for Compilation Errors
```bash
cd backend
cargo check
```

**Expected Output:** `Finished dev [unoptimized + debuginfo] target(s) in X.XXs`

**If you see errors:** Fix them before proceeding!

### Step 6: Run All Tests
```bash
cd backend
cargo test
```

**Expected:** All tests should pass ✅

### Step 7: Run Pagination Tests Specifically
```bash
cd backend
cargo test pagination_tests -- --nocapture
```

**Expected Tests:**
- ✅ `notifications_endpoint_supports_page_and_limit`
- ✅ `admin_logs_endpoint_supports_page_and_limit`
- ✅ `due_plans_endpoint_supports_page_and_limit`
- ✅ `create_plan_accepts_query_pagination_params`

### Step 8: Run Formatting Check
```bash
cd backend
cargo fmt --check
```

### Step 9: Run Linter
```bash
cd backend
cargo clippy -- -D warnings
```

## Common Issues & Fixes:

### Issue 1: "cargo: command not found"
**Fix:** Install Rust/Cargo (see Step 1)

### Issue 2: "database connection failed"
**Fix:** 
- Start PostgreSQL: `sudo service postgresql start`
- Check DATABASE_URL in .env
- Verify credentials

### Issue 3: "migration error"
**Fix:** 
```bash
cd backend
sqlx database create
sqlx migrate run
```

### Issue 4: Compilation errors
**Fix:** Read the error message carefully and fix the code

### Issue 5: Test failures
**Fix:** 
- Check test output for specific failures
- Verify test database is set up correctly
- Ensure test data is being created properly

## Alternative: Use GitHub Codespaces

If you can't install Cargo locally:

1. Go to your GitHub repository
2. Click "Code" → "Codespaces" → "Create codespace"
3. Wait for environment to load (Rust should be pre-installed)
4. Run tests in the codespace terminal
5. Commit and push from codespace

## What Happens If You Skip Testing?

### ❌ Your PR Will Likely:
1. **Fail CI/CD** - Automated tests will fail
2. **Get Rejected** - Reviewers won't review failing PRs
3. **Waste Time** - You'll have to fix issues and re-submit
4. **Block Others** - If merged broken, it breaks the main branch

### ✅ With Testing:
1. **Pass CI/CD** - Automated checks pass
2. **Quick Review** - Reviewers can focus on code quality
3. **Merge Faster** - No back-and-forth fixing issues
4. **Professional** - Shows you care about code quality

## Files Changed (for PR Description):

### New Files:
- `backend/src/pagination.rs` - Core pagination module
- `backend/PAGINATION_CHANGES_SUMMARY.md` - Implementation summary
- `backend/TEST_INSTRUCTIONS.md` - Testing guide
- `backend/PR_CHECKLIST.md` - This checklist

### Modified Files:
- `backend/src/lib.rs` - Added pagination module
- `backend/src/app.rs` - Updated 6 endpoints with pagination
- `backend/src/service.rs` - Added paginated methods
- `backend/src/notifications.rs` - Added paginated methods
- `backend/src/loan_lifecycle.rs` - Added paginated methods + pagination fields to filters
- `backend/tests/pagination_tests.rs` - Uncommented and updated tests

## PR Title Suggestion:
```
feat: Add pagination to list endpoints for performance improvement
```

## PR Description Template:
```markdown
## Description
Implements pagination for all list endpoints to address performance issues with large datasets.

## Changes
- ✅ Created pagination module with offset and cursor-based support
- ✅ Updated 6 core endpoints with pagination
- ✅ Added pagination metadata to responses
- ✅ Default: 20 items/page, Max: 100 items/page
- ✅ Updated and uncommented pagination tests

## Endpoints Updated
1. GET /api/notifications
2. GET /api/admin/logs
3. GET /api/plans/due-for-claim
4. GET /api/admin/plans/due-for-claim
5. GET /api/emergency/contacts
6. GET /api/loans/lifecycle

## Testing
- [x] All tests pass locally
- [x] Pagination tests pass
- [x] No compilation errors
- [x] Cargo fmt and clippy pass

## Breaking Changes
Response format now includes pagination metadata. Endpoints work without pagination params (use defaults).

## Related Issue
Closes #XXX (replace with actual issue number)
```

## Final Checklist Before PR:

- [ ] Cargo is installed
- [ ] All tests pass (`cargo test`)
- [ ] Pagination tests pass (`cargo test pagination_tests`)
- [ ] No compilation errors (`cargo check`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Database migrations run successfully
- [ ] Manual API testing done (optional but recommended)
- [ ] PR description is complete
- [ ] Commit messages are clear

## If All Checks Pass:

```bash
git add .
git commit -m "feat: add pagination to list endpoints"
git push origin your-branch-name
```

Then create your PR on GitHub! 🚀

## Need Help?

If you're stuck:
1. Read the error messages carefully
2. Check `backend/TEST_INSTRUCTIONS.md`
3. Check `backend/PAGINATION_CHANGES_SUMMARY.md`
4. Ask in the PR comments or issue tracker
