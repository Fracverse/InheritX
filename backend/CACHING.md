# API Response Caching Strategy

## Overview

InheritX uses **pure HTTP caching** via ETags and `Cache-Control` headers.  
There is **no shared cache store** (no Redis, no Varnish) — caching is handled entirely by the HTTP protocol between the server and each client.

This approach requires zero additional infrastructure and is fully stateless.

---

## How It Works

### ETag Generation

Every cacheable GET response includes an `ETag` header computed as:

```
ETag = Base64URL( SHA-256( canonical_json(response_body) ) )
```

- The ETag changes automatically whenever the underlying data changes.
- It is a **strong ETag** (no `W/` prefix), so byte-for-byte equality is required.

### Conditional Requests (If-None-Match)

Clients that have previously received a response **should** store the `ETag` and send it on subsequent requests:

```
GET /api/plans/abc-123
If-None-Match: "xYz9..."
```

If the ETag still matches (data has not changed), the server returns:

```
HTTP/1.1 304 Not Modified
ETag: "xYz9..."
Cache-Control: private, max-age=60, must-revalidate
```

The response body is empty, saving bandwidth and reducing latency.

If the data has changed, the server returns a normal `200 OK` with the new ETag.

---

## Cached Endpoints

| Endpoint | Method | Cache-Control | Notes |
|---|---|---|---|
| `/health` | GET | `public, max-age=10, must-revalidate` | Polled frequently; body is constant |
| `/api/plans/:plan_id` | GET | `private, max-age=60, must-revalidate` | User-specific; changes on plan updates |
| `/api/loans/simulations` | GET | `private, max-age=120, must-revalidate` | Changes when new simulations are created |
| `/api/loans/simulations/:id` | GET | `private, max-age=300, must-revalidate` | Simulations are immutable once saved |
| `/api/reputation` | GET | `private, max-age=120, must-revalidate` | Changes on loan events |
| `/api/loans/lifecycle/summary` | GET | `private, max-age=60, must-revalidate` | Changes on any loan state transition |
| `/api/loans/lifecycle/:id` | GET | `private, max-age=60, must-revalidate` | Changes on repay/liquidate |
| `/api/governance/proposals` | GET | `public, max-age=120, must-revalidate` | Rarely changes |

### `public` vs `private`

- **`public`**: Response may be stored by shared caches (CDN, reverse proxy). Used only for non-user-specific data.
- **`private`**: Response must only be stored by the user's browser/client. Used for all user-authenticated data.

---

## Cache Invalidation

### Automatic (ETag-based)

Because the ETag is a hash of the response body, it **changes automatically** whenever the underlying data changes. No explicit invalidation is needed — the client simply revalidates on the next request.

Example flow:
1. `GET /api/plans/abc` → `200 OK`, `ETag: "AAA"`, `Cache-Control: private, max-age=60`
2. Client caches the response for 60 seconds.
3. After 60 seconds, client sends `GET /api/plans/abc` with `If-None-Match: "AAA"`.
4. If the plan was **not updated**: server returns `304`, client uses cached copy.
5. If the plan **was updated**: server returns `200` with new body and new `ETag: "BBB"`.

### Write Endpoints (no-store)

All mutating requests (POST, PUT, PATCH, DELETE) receive:

```
Cache-Control: no-store
```

This is applied automatically by the `cache_headers_middleware` layer. Write responses are **never** stored in any cache.

---

## Adding Caching to a New Endpoint

1. Add `headers: HeaderMap` as a parameter to the handler function.
2. Fetch your data from the database as normal.
3. Serialize to a `serde_json::Value` body.
4. Apply the caching pattern:

```rust
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use crate::cache;

async fn my_get_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let data = MyService::fetch(&state.db).await?;
    let body = json!({ "status": "success", "data": data });

    let etag = cache::compute_etag(&body);
    if cache::is_not_modified(&headers, &etag) {
        return Ok(cache::not_modified_response_with_cc(
            &etag,
            cache::cache_control_private(60),  // or cache_control_public(N)
        ));
    }

    let mut response = Json(body).into_response();
    cache::apply_cache_headers(&mut response, &etag, cache::cache_control_private(60));
    Ok(response)
}
```

5. Choose the right `Cache-Control` policy:
   - User-specific data → `cache_control_private(max_age_secs)`
   - Public/shared data → `cache_control_public(max_age_secs)`
   - Never cache → don't apply; the middleware sets `no-store` on writes automatically.

---

## Implementation Files

| File | Purpose |
|---|---|
| `src/cache.rs` | Core primitives: ETag computation, header builders, response helpers |
| `src/middleware.rs` | `cache_headers_middleware` — stamps `no-store` on all write responses |
| `src/app.rs` | Middleware wiring + per-handler ETag logic |
| `tests/cache_tests.rs` | Unit + integration tests |

---

## Testing Caching Behaviour

### With curl

```bash
# 1. First request — expect 200 with ETag header
curl -si http://localhost:8080/health

# 2. Conditional request — expect 304
ETAG=$(curl -si http://localhost:8080/health | grep -i etag | awk '{print $2}' | tr -d '\r')
curl -si -H "If-None-Match: $ETAG" http://localhost:8080/health

# 3. Write endpoint — expect no-store
curl -si -X POST http://localhost:8080/api/plans -H "Content-Type: application/json" -d '{...}'
```

### Automated

```powershell
cd backend
cargo test cache
```
