# Data Retention Policy

This document defines backend data lifecycle controls for Issue #453.

## Retention Periods

- notifications table: 180 days, then archived into data_archives and deleted.
- action_logs table: 365 days, then archived into data_archives and deleted.
- sessions table: 30 days, then deleted.

## Automated Archival

- Worker: DataRetentionService::start_archive_worker
- Interval config: RETENTION_ARCHIVE_INTERVAL_SECS (default: 3600)
- Manual run endpoint: POST /api/admin/data-retention/archive/run

## GDPR / Right To Be Forgotten

- Endpoint: DELETE /api/user/data
- Modes:
  - hard_delete=true: user row deleted (cascading foreign keys apply)
  - hard_delete=false (default): PII anonymized and linked records cleaned

## User Data Export

- Endpoint: GET /api/user/data-export
- Includes:
  - users row
  - plans rows
  - notifications rows
  - action_logs rows

## Archive Storage

Archived records are persisted in data_archives with:

- source_table
- source_id
- payload (jsonb)
- archived_at timestamp
