#!/usr/bin/env bash
# =============================================================================
# InheritX Backup Monitoring Script
# =============================================================================
# Verifies that a recent backup exists and alerts if the newest backup is
# older than the configured threshold.  Intended to run shortly after the
# scheduled backup window (e.g. 03:30 UTC if backups run at 02:30 UTC).
#
# Required environment variables:
#   BACKUP_DIR              – directory where backups are stored (default: /backups)
#
# Optional environment variables:
#   BACKUP_MAX_AGE_HOURS    – alert if newest backup is older than this (default: 25)
#   ALERT_WEBHOOK_URL       – Slack / PagerDuty / generic webhook URL for alerts
#   ALERT_EMAIL             – email address to notify on failure (requires sendmail)
#
# Exit codes:
#   0  – backup is recent
#   1  – no backup found or backup too old
# =============================================================================

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
MAX_AGE_HOURS="${BACKUP_MAX_AGE_HOURS:-25}"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
alert() {
    local message="$1"
    log "ALERT: ${message}"

    if [[ -n "${ALERT_WEBHOOK_URL:-}" ]]; then
        curl -s -X POST "${ALERT_WEBHOOK_URL}" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"[InheritX Backup Alert] ${message}\"}" || true
    fi

    if [[ -n "${ALERT_EMAIL:-}" ]] && command -v sendmail &>/dev/null; then
        echo -e "Subject: [InheritX] Backup Alert\n\n${message}" \
            | sendmail "${ALERT_EMAIL}" || true
    fi
}

log "=== InheritX backup monitor starting ==="

# Find the most recent backup file
NEWEST="$(find "${BACKUP_DIR}" -name "inheritx-backup-*.sql.gz*" \
    -type f -printf "%T@ %p\n" 2>/dev/null \
    | sort -n | tail -1 | cut -d' ' -f2)"

if [[ -z "${NEWEST}" ]]; then
    alert "No backup files found in ${BACKUP_DIR}"
    exit 1
fi

# Check age
MTIME_EPOCH="$(stat -c %Y "${NEWEST}")"
NOW_EPOCH="$(date +%s)"
AGE_HOURS=$(( (NOW_EPOCH - MTIME_EPOCH) / 3600 ))
BACKUP_NAME="$(basename "${NEWEST}")"

log "Newest backup: ${BACKUP_NAME} (${AGE_HOURS}h old)"

if (( AGE_HOURS > MAX_AGE_HOURS )); then
    alert "Newest backup '${BACKUP_NAME}' is ${AGE_HOURS}h old, exceeds threshold of ${MAX_AGE_HOURS}h. Check backup job immediately."
    exit 1
fi

log "Backup is within acceptable age (${AGE_HOURS}h ≤ ${MAX_AGE_HOURS}h). All OK."
log "=== Backup monitor complete ==="
