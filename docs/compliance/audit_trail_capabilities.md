# Audit Trail Capabilities

This document details the audit trail capabilities of the InheritX platform, ensuring compliance with SOC2 and other regulatory frameworks requiring strict traceability.

## Event Logging
The system logs the following events immutably:
- All authentication attempts (successful and failed).
- Access to sensitive data (e.g., viewing a will or emergency contact).
- State changes to core entities (e.g., Plan creation, Loan repayment).
- Administrative actions (e.g., overriding risk parameters, granting emergency access).

## Log Structure
Each audit log entry contains:
- **Timestamp:** ISO 8601 UTC timestamp.
- **Actor:** UUID of the user or service account performing the action.
- **Action:** A standardized string identifying the action.
- **Resource:** UUID of the affected resource.
- **Context:** JSON payload with additional metadata (e.g., IP address, user-agent).

## Log Protection
- Audit logs are stored in an append-only database table.
- A background worker periodically exports audit logs to WORM (Write Once, Read Many) storage to prevent tampering.
- Access to audit logs is strictly limited to authorized compliance and security personnel.
