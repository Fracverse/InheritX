#!/usr/bin/env bash
# =============================================================================
# InheritX PostgreSQL Backup Restoration Script
# =============================================================================
# Restores the INHERITX database from a compressed pg_dump file.
#
# Usage:
#   ./restore.sh <backup-file>
#
# Required environment variables:
#   RESTORE_DATABASE_URL  – target PostgreSQL connection string
#
# Optional environment variables:
#   RESTORE_DROP_DB       – set to "yes" to DROP and re-CREATE the target
#                           database before restoring (default: "no").
#                           CAUTION: all existing data will be lost.
#   RESTORE_GPG_PASSPHRASE – passphrase if the backup file is GPG-encrypted
#                            (.gpg extension expected).
#
# Exit codes:
#   0  – success
#   1  – pre-flight check failed
#   2  – restore failed
# =============================================================================

set -euo pipefail

BACKUP_FILE="${1:-}"
log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

log "=== InheritX restore starting ==="

# ── Pre-flight ────────────────────────────────────────────────────────────────
if [[ -z "${BACKUP_FILE}" ]]; then
    log "ERROR: Usage: $0 <backup-file>"
    exit 1
fi

if [[ ! -f "${BACKUP_FILE}" ]]; then
    log "ERROR: Backup file not found: ${BACKUP_FILE}"
    exit 1
fi

if [[ -z "${RESTORE_DATABASE_URL:-}" ]]; then
    log "ERROR: RESTORE_DATABASE_URL is not set"
    exit 1
fi

if ! command -v pg_restore &>/dev/null; then
    log "ERROR: pg_restore not found in PATH"
    exit 1
fi

# ── Decrypt if needed ─────────────────────────────────────────────────────────
WORK_FILE="${BACKUP_FILE}"
if [[ "${BACKUP_FILE}" == *.gpg ]]; then
    if [[ -z "${RESTORE_GPG_PASSPHRASE:-}" ]]; then
        log "ERROR: Backup file is encrypted but RESTORE_GPG_PASSPHRASE is not set"
        exit 1
    fi
    DECRYPTED="/tmp/inheritx-restore-$(date +%s).sql.gz"
    log "Decrypting ${BACKUP_FILE} to ${DECRYPTED}"
    gpg --batch --yes --passphrase "${RESTORE_GPG_PASSPHRASE}" \
        --output "${DECRYPTED}" --decrypt "${BACKUP_FILE}"
    WORK_FILE="${DECRYPTED}"
    # shellcheck disable=SC2064
    trap "rm -f '${DECRYPTED}'" EXIT
fi

# ── Optional: drop and recreate target database ────────────────────────────────
if [[ "${RESTORE_DROP_DB:-no}" == "yes" ]]; then
    # Extract dbname from the URL for the confirmation prompt
    DB_NAME="${RESTORE_DATABASE_URL##*/}"
    log "WARNING: About to DROP database '${DB_NAME}'. Sleeping 5s – hit Ctrl-C to abort."
    sleep 5
    log "Dropping and recreating database ${DB_NAME}"
    # Connect to 'postgres' maintenance DB to drop/create
    ADMIN_URL="${RESTORE_DATABASE_URL%/*}/postgres"
    psql "${ADMIN_URL}" -c "DROP DATABASE IF EXISTS \"${DB_NAME}\";"
    psql "${ADMIN_URL}" -c "CREATE DATABASE \"${DB_NAME}\";"
fi

# ── Restore ───────────────────────────────────────────────────────────────────
log "Restoring from ${WORK_FILE} to ${RESTORE_DATABASE_URL}"
if ! gunzip -c "${WORK_FILE}" | pg_restore \
    --no-password \
    --clean \
    --if-exists \
    --dbname "${RESTORE_DATABASE_URL}"; then
    log "ERROR: pg_restore failed"
    exit 2
fi

log "=== Restore complete ==="
log "RTO validation: run 'cargo test' against the restored database to verify integrity."
