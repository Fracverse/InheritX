# InheritX PostgreSQL Backup & Disaster Recovery Runbook

## Overview

This document describes the automated backup strategy, retention policy,
restoration procedure, and disaster-recovery (DR) plan for the InheritX
PostgreSQL database.

---

## 1. Backup Strategy

### 1.1 Automated Daily Backups

A `pg_dump` (custom format, compressed) is taken every day by the script
`backend/scripts/backup.sh`.  The script is invoked by the environment's
scheduler (cron, Kubernetes CronJob, AWS ECS Scheduled Task, etc.).

**Recommended schedule:** `30 2 * * *` (02:30 UTC – low-traffic window)

**Required environment variable:**

| Variable | Description |
|---|---|
| `BACKUP_DATABASE_URL` | Full PostgreSQL connection string |

**Optional environment variables:**

| Variable | Default | Description |
|---|---|---|
| `BACKUP_DIR` | `/backups` | Local directory to store dump files |
| `BACKUP_RETENTION_DAYS` | `30` | Days of local backup files to keep |
| `BACKUP_S3_BUCKET` | _(unset)_ | S3 bucket prefix for off-site storage |
| `BACKUP_ENCRYPT_KEY` | _(unset)_ | GPG recipient key ID for encryption at rest |

### 1.2 Point-in-Time Recovery (PITR)

WAL archiving must be enabled on the PostgreSQL instance to support PITR:

```sql
-- postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'aws s3 cp %p s3://my-bucket/inheritx-wal/%f'
restore_command = 'aws s3 cp s3://my-bucket/inheritx-wal/%f %p'
```

With WAL archiving active, the database can be restored to any point in time
between two daily base backups with RPO < 1 minute.

**Managed service equivalents:**
- AWS RDS: enable "Automated Backups" + set backup retention window.
- Supabase: enable PITR in project settings.
- DigitalOcean Managed PostgreSQL: enable daily backups with PITR add-on.

---

## 2. Retention Policy

| Backup type | Retention | Storage location |
|---|---|---|
| Daily compressed dump | 30 days | Local **and** S3 (`STANDARD_IA`) |
| WAL archive files | 7 days | S3 (`STANDARD_IA`) |
| Monthly snapshots | 12 months | S3 (`GLACIER`) |

Monthly snapshots are created by tagging the first backup of each month in the
lifecycle policy of the S3 bucket:

```json
{
  "Rules": [
    {
      "Id": "inheritx-daily-backups",
      "Filter": { "Prefix": "inheritx-backups/" },
      "Status": "Enabled",
      "Transitions": [
        { "Days": 30, "StorageClass": "GLACIER" }
      ],
      "Expiration": { "Days": 365 }
    }
  ]
}
```

---

## 3. Backup Monitoring & Alerts

The monitoring script `backend/scripts/backup_monitor.sh` verifies that a
recent backup exists.  Run it 30–60 minutes after the scheduled backup window.

**Recommended schedule:** `00 4 * * *` (04:00 UTC)

| Variable | Default | Description |
|---|---|---|
| `BACKUP_DIR` | `/backups` | Directory to scan for backup files |
| `BACKUP_MAX_AGE_HOURS` | `25` | Alert if newest backup is older than this |
| `ALERT_WEBHOOK_URL` | _(unset)_ | Slack / PagerDuty / generic webhook |
| `ALERT_EMAIL` | _(unset)_ | Email address for failure notifications |

The script exits non-zero on failure; integrate it with your alerting stack
(e.g. wrap in a Prometheus pushgateway push or a CloudWatch alarm).

---

## 4. Restoration Procedure

Use `backend/scripts/restore.sh` to restore a backup.

### 4.1 Full restore to a clean database

```bash
export RESTORE_DATABASE_URL="postgres://user:pass@host:5432/inheritx"
export RESTORE_DROP_DB=yes           # drops and re-creates the target DB
./backend/scripts/restore.sh /backups/inheritx-backup-20260424T023012Z.sql.gz
```

### 4.2 Restore an encrypted backup

```bash
export RESTORE_DATABASE_URL="postgres://..."
export RESTORE_GPG_PASSPHRASE="your-gpg-passphrase"
./backend/scripts/restore.sh /backups/inheritx-backup-20260424T023012Z.sql.gz.gpg
```

### 4.3 Restore to a staging environment for validation

```bash
export RESTORE_DATABASE_URL="postgres://user:pass@staging-host:5432/inheritx_staging"
./backend/scripts/restore.sh /backups/inheritx-backup-20260424T023012Z.sql.gz
# Then run integration tests against staging
cd backend && DATABASE_URL="${RESTORE_DATABASE_URL}" cargo test
```

---

## 5. Restoration Testing

Backup restoration **must** be tested regularly.  Untested backups are
worthless.

| Frequency | Action |
|---|---|
| **Weekly** | Automated restore to isolated staging environment; run `cargo test` to verify schema and data integrity. |
| **Monthly** | Manual DR drill: restore entire stack from scratch in a clean environment; verify RTO. |
| **Quarterly** | Full DR exercise: simulate production data loss; measure actual RTO and RPO; update this document. |

---

## 6. Recovery Time Objective (RTO) & Recovery Point Objective (RPO)

| Scenario | Target | Notes |
|---|---|---|
| Single table corruption | < 30 min | Restore from latest dump to staging; copy table |
| Full database loss (with PITR) | **< 4 hours** | Restore base backup + replay WAL |
| Full database loss (daily dump only) | < 4 hours | Restore last dump; up to 24 h data loss |
| Complete environment loss | < 8 hours | Provision new infra + restore DB + redeploy app |

**RPO (maximum data loss):**
- With WAL archiving: < 1 minute
- Without WAL archiving: up to 24 hours (since last dump)

---

## 7. Disaster Recovery Runbook

### Step 1 – Declare the incident

1. Page on-call engineer via PagerDuty / Slack `#incidents` channel.
2. Open a war-room in Slack `#dr-active`.
3. Assign incident commander.

### Step 2 – Assess damage

- Determine what data / services are affected.
- Check monitoring dashboards for the time of first anomaly.
- Identify the last known-good backup.

### Step 3 – Provision target environment (if needed)

```bash
# Example: spin up a new RDS instance via Terraform
cd infra/
terraform apply -target=aws_db_instance.inheritx_primary
```

### Step 4 – Restore database

```bash
# Download latest backup from S3 if local copy is unavailable
aws s3 cp s3://my-bucket/inheritx-backups/<backup-file> /tmp/

export RESTORE_DATABASE_URL="postgres://..."
export RESTORE_DROP_DB=yes
./backend/scripts/restore.sh /tmp/<backup-file>
```

### Step 5 – Validate restoration

```bash
cd backend
DATABASE_URL="${RESTORE_DATABASE_URL}" cargo test
```

### Step 6 – Redirect traffic

Update DNS / load-balancer to point at the restored instance.
Confirm health-check endpoints return 200:

```bash
curl https://api.inheritx.io/health
curl https://api.inheritx.io/health/db
```

### Step 7 – Post-incident review

Within 48 hours of resolution:
- Document timeline, root cause, and resolution steps.
- Update RTO/RPO measurements.
- Identify process improvements.

---

## 8. Quick Reference

```
# Take a manual backup right now
BACKUP_DATABASE_URL=postgres://... ./backend/scripts/backup.sh

# Check if backups are healthy
BACKUP_DIR=/backups ./backend/scripts/backup_monitor.sh

# Restore from a specific backup
RESTORE_DATABASE_URL=postgres://... ./backend/scripts/restore.sh /backups/<file>
```
