# Sync Fork with Upstream - Resolve PR Conflicts

PR #153 has merge conflicts because the fork (`zintarh/InheritX`) has diverged from upstream (`Fracverse/InheritX`). The fork's `master` branch uses a different backend architecture (deadpool_postgres) while upstream uses sqlx/PgPool.

## Quick Fix - Run Locally

From your InheritX clone:

```bash
# 1. Add upstream remote
git remote add upstream https://github.com/Fracverse/InheritX.git  # if not already added

# 2. Fetch and sync
git fetch upstream
git checkout master
git merge upstream/master
git push origin master

# 3. Rebase feature branch
git checkout feat/kyc-access-control
git rebase master

# 4. Resolve any conflicts (edit files, then:)
git add .
git rebase --continue

# 5. Force push
git push origin feat/kyc-access-control --force-with-lease
```

Or run the script:
```bash
chmod +x scripts/sync-upstream-and-resolve-conflicts.sh
./scripts/sync-upstream-and-resolve-conflicts.sh
```

## Alternative: Create Fresh Branch from Upstream

If rebase has too many conflicts:

```bash
git fetch upstream
git checkout -b feat/kyc-access-control-v2 upstream/master
# Then cherry-pick or manually apply the KYC changes from feat/kyc-access-control
git push origin feat/kyc-access-control-v2
# Create new PR from feat/kyc-access-control-v2, close #153
```

## What Our PR Changes

- `backend/src/app.rs`: KYC validation with specific error messages
- `backend/tests/kyc_access.rs`: Enhanced integration tests
- `backend/tests/helpers/mod.rs`: Test helpers
