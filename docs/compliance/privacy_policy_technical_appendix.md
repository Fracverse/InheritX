# Privacy Policy Technical Appendix

This technical appendix details the specific mechanisms employed by InheritX to protect user privacy in accordance with GDPR and other relevant privacy regulations.

## Anonymization and Pseudonymization
- Sensitive data fields are pseudonymized using secure hashing algorithms (e.g., SHA-256 with salts) before being used in analytical workloads.
- Personally Identifiable Information (PII) is stored in encrypted databases separate from transaction records.

## Data Subject Access Requests (DSAR)
- Automated API endpoints are available to export a user's data in a machine-readable format (JSON).
- Deletion requests trigger a soft delete, followed by a hard delete and cryptographic erasure after the mandatory 30-day retention period.

## Consent Management
- User consent for data processing is tracked immutably within the database.
- Opt-out mechanisms automatically propagate to all integrated third-party systems.
