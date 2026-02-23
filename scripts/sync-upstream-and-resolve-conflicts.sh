#!/bin/bash
# Sync fork with upstream and resolve PR #153 conflicts
# Run this from the InheritX repo root

set -e

echo "=== InheritX Fork Sync Script ==="
echo "This script will:"
echo "  1. Add upstream remote (Fracverse/InheritX)"
echo "  2. Fetch upstream master"
echo "  3. Sync fork's master with upstream"
echo "  4. Rebase feat/kyc-access-control onto synced master"
echo "  5. Force-push the rebased branch"
echo ""

# Check we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository. Run from InheritX root."
    exit 1
fi

# Add upstream if not present
if ! git remote get-url upstream > /dev/null 2>&1; then
    echo "Adding upstream remote..."
    git remote add upstream https://github.com/Fracverse/InheritX.git
fi

echo "Fetching upstream..."
git fetch upstream

echo "Syncing fork's master with upstream/master..."
git checkout master
git merge upstream/master --no-edit

echo "Pushing updated master to fork..."
git push origin master

echo "Rebasing feat/kyc-access-control onto master..."
git checkout feat/kyc-access-control
git rebase master

echo "Force-pushing rebased branch..."
git push origin feat/kyc-access-control --force-with-lease

echo ""
echo "=== Done! ==="
echo "PR #153 should now be conflict-free. Check: https://github.com/Fracverse/InheritX/pull/153"