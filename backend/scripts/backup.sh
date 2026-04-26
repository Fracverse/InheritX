#!/usr/bin/env bash
# =============================================================================
# InheritX PostgreSQL Backup Script
# =============================================================================
# Creates a compressed, timestamped pg_dump of the INHERITX database.
# Designed to be run daily by cron or a container scheduler.
#
# Required environment variables:
#   BACKUP_DATABASE_URL   – full PostgreSQL connection string
#                           e.g. postgres://user:pass@host:5432/inheritx
#
# Optional environment variables:
#   BACKUP_DIR            – local directory to store backups (default: /backups)
#   BACKUP_RETENTION_DAYS – how many days of local backups to keep (default: 30)
#   BACKUP_S3_BUCKET      – if set, the compressed dump is uploaded to this S3
#                           bucket prefix (e.g. s3://my-bucket/inheritx-backups)
#                           Requires the AWS CLI and IAM credentials in the env.
#   BACKUP_ENCRYPT_KEY    – if set, the dump is GPG-encrypted with this
#                           recipient key ID before upload.
#
# Exit codes:
#   0  – success
#   1  – pre-flight check failed (missing variable / tool)
#   2  – pg_dump failed
#   3  – S3 upload failed
#
# Cron example (daily at 02:30 UTC):
#   30 2 * * * /opt/inheritx/scripts/backup.sh >> /var/log/inheritx-backup.log 2>&1
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
BACKUP_DIR="${BACKUP_DIR:-/backups}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_FILENAME="inheritx-backup-${TIMESTAMP}.sql.gz"
BACKUP_PATH="${BACKUP_DIR}/${BACKUP_FILENAME}"

# ── Pre-flight checks ─────────────────────────────────────────────────────────
log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

log "=== InheritX backup starting ==="

if [[ -z "${BACKUP_DATABASE_URL:-}" ]]; then
    log "ERROR: BACKUP_DATABASE_URL is not set"
    exit 1
fi

if ! command -v pg_dump &>/dev/null; then
    log "ERROR: pg_dump not found in PATH"
    exit 1
fi

mkdir -p "${BACKUP_DIR}"

# ── Create backup ─────────────────────────────────────────────────────────────
log "Dumping database to ${BACKUP_PATH}"
if ! pg_dump \
    --format=custom \
    --compress=9 \
    --no-password \
    "${BACKUP_DATABASE_URL}" \
    | gzip -9 > "${BACKUP_PATH}"; then
    log "ERROR: pg_dump failed"
    exit 2
fi

BACKUP_SIZE="$(du -sh "${BACKUP_PATH}" | cut -f1)"
log "Backup created: ${BACKUP_PATH} (${BACKUP_SIZE})"

# ── Optional: GPG encryption ──────────────────────────────────────────────────
if [[ -n "${BACKUP_ENCRYPT_KEY:-}" ]]; then
    log "Encrypting backup with key ${BACKUP_ENCRYPT_KEY}"
    gpg --batch --yes --recipient "${BACKUP_ENCRYPT_KEY}" \
        --output "${BACKUP_PATH}.gpg" \
        --encrypt "${BACKUP_PATH}"
    rm -f "${BACKUP_PATH}"
    BACKUP_PATH="${BACKUP_PATH}.gpg"
    log "Encrypted backup: ${BACKUP_PATH}"
fi

# ── Optional: S3 upload ───────────────────────────────────────────────────────
if [[ -n "${BACKUP_S3_BUCKET:-}" ]]; then
    if ! command -v aws &>/dev/null; then
        log "ERROR: aws CLI not found but BACKUP_S3_BUCKET is set"
        exit 3
    fi
    S3_KEY="${BACKUP_S3_BUCKET}/$(basename "${BACKUP_PATH}")"
    log "Uploading to ${S3_KEY}"
    if ! aws s3 cp "${BACKUP_PATH}" "${S3_KEY}" \
            --storage-class STANDARD_IA \
            --metadata "timestamp=${TIMESTAMP},source=inheritx-backup-script"; then
        log "ERROR: S3 upload failed"
        exit 3
    fi
    log "Upload complete: ${S3_KEY}"
fi

# ── Prune old local backups ───────────────────────────────────────────────────
log "Pruning local backups older than ${RETENTION_DAYS} days"
find "${BACKUP_DIR}" -name "inheritx-backup-*.sql.gz*" \
    -mtime "+${RETENTION_DAYS}" -delete -print | while read -r f; do
    log "Deleted old backup: ${f}"
done

log "=== Backup complete ==="
