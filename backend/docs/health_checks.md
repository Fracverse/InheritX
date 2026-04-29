# Backend Health Check Endpoints

The InheritX backend provides the following health check endpoints for use by load balancers, orchestrators (e.g., Kubernetes), and monitoring systems.

## Liveness Probe

**Endpoint:** `GET /health`

This endpoint serves as a simple liveness probe to verify that the HTTP server is running and able to accept connections.

**Response:**
Returns `200 OK` on success.

```json
{
  "status": "ok",
  "message": "App is running",
  "service": "inheritx-backend"
}
```

## Readiness Probe

**Endpoint:** `GET /ready`

This endpoint serves as a readiness probe to verify that the application and its dependencies (e.g., the database) are fully initialized and ready to process traffic.

**Response:**
Returns `200 OK` on success, or `500 Internal Server Error` if a dependency is unavailable.

```json
{
  "status": "ok",
  "database": {
    "status": "ok",
    "latency_ms": 12
  }
}
```

## Database Health Metrics

**Endpoint:** `GET /health/db`

Returns detailed information about the database connection pool, including latency and utilization.

**Endpoint:** `GET /health/db/metrics`

Returns raw metrics for Prometheus scraping.
