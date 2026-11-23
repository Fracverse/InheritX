-- Issue #1274: metadata for KYC evidence files persisted by
-- POST /api/kyc/upload. The bytes live in KYC_STORAGE_DIR; this table records
-- ownership and the attributes needed to serve the object back.

CREATE TABLE IF NOT EXISTS kyc_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_address TEXT NOT NULL,
    document_type TEXT NOT NULL
        CHECK (document_type IN ('id', 'address_proof', 'selfie')),
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL CHECK (size_bytes > 0),
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS kyc_documents_owner_address_idx
    ON kyc_documents (owner_address);
