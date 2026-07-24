@echo off
cd /d "%~dp0"
git checkout -b fix/secure-jwt-auth-945
git add backend/src/auth.rs PR_DESCRIPTION.md
git commit -m "feat(auth): enforce minimum JWT secret length and explicit expiry validation"
git push origin fix/secure-jwt-auth-945
gh pr create --repo scarface-dev1/InheritX --title "feat(auth): enforce minimum JWT secret length and explicit expiry validation" --body-file "%~dp0PR_DESCRIPTION.md"