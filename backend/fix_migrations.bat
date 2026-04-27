@echo off
REM Fix all migrations to include uuid-ossp extension

echo Fixing migrations to include uuid-ossp extension...

REM List of migrations that need the fix (excluding init)
set migrations=20260221151200_add_notifications_and_action_logs.sql 20260224153500_add_nonces.sql 20260226000000_add_lending_events.sql 20260226140000_create_user_2fa.sql 20260324120000_add_emergency_access_tracking.sql 20260324170000_add_emergency_contacts.sql 20260324173000_add_emergency_access_audit_logs.sql 20260324180000_add_emergency_access_risk_alerts.sql 20260325100000_add_pools_for_stress_testing.sql 20260325103000_add_governance_tables.sql 20260325190000_add_emergency_access_sessions.sql

for %%f in (%migrations%) do (
    echo Checking migrations\%%f
    findstr /C:"CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"" migrations\%%f >nul
    if errorlevel 1 (
        echo Adding extension to %%f
        echo -- Ensure UUID extension is available> temp.sql
        echo CREATE EXTENSION IF NOT EXISTS "uuid-ossp";>> temp.sql
        echo.>> temp.sql
        type migrations\%%f >> temp.sql
        move /Y temp.sql migrations\%%f >nul
    ) else (
        echo %%f already has extension
    )
)

echo Done!
