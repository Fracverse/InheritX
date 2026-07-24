# Secure JWT token generator

Closes #36

## Summary
This PR hardens the backend JWT issuance and validation path so tokens are generated and verified using stricter security rules.

## Problem
- Short or empty `JWT_SECRET` values were accepted in code paths, reducing resistance to brute-force attacks.
- Tokens lacked a unique identifier (`jti`) and `not_before` discipline.
- Expiry handling relied only on parser return values; validation could be tighter and more explicit.

## Solution
- **Enforce minimum JWT secret length at boundaries**:
  - `IssueJWT()` rejects secrets shorter than 32 characters.
  - `ParseJWT()` rejects secrets shorter than 32 characters before signing-method checks.
- **Add token uniqueness**: `IssueJWT()` generates a random `jti` for each token.
- **Prevent early use**: `NotBefore` is set to issue time (`iat`).
- **Keep algorithm restriction**: only HS256 is accepted; reject `none`, RS256, and other algorithms.
- **Expiry behavior**: existing tests confirm parsing fails exactly at or after expiration.

## Files changed
- `internal/auth/jwt.go`
  - Enforce `len(secret) >= 32` on issue and parse.
  - Add `jti` via `randomTokenID()`.
  - Set `NotBefore` to issue time.
  - Keep HS256-only enforcement and explicit expiry error wrapping.

- `internal/auth/jwt_test.go`
  - Use long secrets in tests to match new runtime checks.
  - Preserve expiry boundary assertions and invalid-token cases.

- `internal/auth/middleware_test.go`
  - Introduce a long shared test secret for middleware routes.
  - Update role/route tests to use the new minimum-length secret.

## Local verification
```bash
cd Grainlify-Backend
go build ./...
go vet ./...
go test ./internal/auth/... -count=1
```

Auth package passes. Full suite includes unrelated failures outside this change; the auth package coverage and assertions remain intact.

## Security considerations
- Minimum 32-character secret requirement reduces exposure to weak secrets.
- Unique `jti` supports future revocation or token introspection.
- `not_before` removes clock-skew acceptance windows around issuance.
- HS256-only parsing blocks algorithm-confusion attacks.

## Related
- Issue #36: Secure JWT token generator