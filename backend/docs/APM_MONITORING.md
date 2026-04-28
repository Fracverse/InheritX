# APM Monitoring Guide

Issue #454 implementation uses Sentry tracing plus Prometheus metrics alerts.

## Instrumentation

- Request latency:
  - http_requests_total
  - http_request_duration_seconds
- DB latency:
  - db_query_duration_seconds
- Alert counters:
  - apm_alerts_total{type="endpoint_latency"}
  - apm_alerts_total{type="db_query_latency"}

## Slow Threshold Configuration

- APM_SLOW_ENDPOINT_MS (default: 1500)
- APM_SLOW_DB_QUERY_MS (default: 200)

When thresholds are exceeded:

- warning logs are emitted
- apm_alerts_total is incremented

## Sentry APM

- Tracing is enabled via SENTRY_TRACES_SAMPLE_RATE.
- Sentry environment and release tagging are configured in error_tracking.rs.

## Suggested Dashboard Panels

- p95 endpoint latency by path
- p95 DB latency by operation
- apm_alerts_total by type
- http_requests_total by status

## Suggested Alerts

- endpoint latency p95 > 1.5s for 5m
- db latency p95 > 200ms for 10m
- apm_alerts_total increasing continuously for 10m
