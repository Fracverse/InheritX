-- ──────────────────────────────────────────────────────────────────────────────
-- Will PDF Generator, Template Engine, Beneficiary Sync & Digital Signatures
-- Tasks 1, 2, 3, 4
-- ──────────────────────────────────────────────────────────────────────────────

-- Will documents table (Tasks 1 & 2)
CREATE TABLE will_documents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id         UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template        VARCHAR(50) NOT NULL,
    will_hash       VARCHAR(64) NOT NULL,
    version         INTEGER NOT NULL DEFAULT 1,
    filename        VARCHAR(255) NOT NULL,
    pdf_base64      TEXT NOT NULL,
    generated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_will_documents_plan_id  ON will_documents(plan_id);
CREATE INDEX idx_will_documents_user_id  ON will_documents(user_id);
CREATE INDEX idx_will_documents_version  ON will_documents(plan_id, version DESC);

-- Plan beneficiaries table (Task 3 — sync source of truth)
CREATE TABLE plan_beneficiaries (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id             UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    wallet_address      VARCHAR(255) NOT NULL,
    allocation_percent  NUMERIC(7, 4) NOT NULL CHECK (allocation_percent > 0 AND allocation_percent <= 100),
    name                VARCHAR(255),
    relationship        VARCHAR(100),
    created_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (plan_id, wallet_address)
);

CREATE INDEX idx_plan_beneficiaries_plan_id ON plan_beneficiaries(plan_id);

-- Will signing challenges table (Task 4 — nonce/replay prevention)
CREATE TABLE will_signing_challenges (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES will_documents(id) ON DELETE CASCADE,
    vault_id        VARCHAR(255) NOT NULL,
    wallet_address  VARCHAR(255) NOT NULL,
    message         TEXT NOT NULL,
    message_hash    VARCHAR(64) NOT NULL,
    nonce           VARCHAR(36) NOT NULL,
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
    used            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_will_signing_challenges_document_id ON will_signing_challenges(document_id);
CREATE INDEX idx_will_signing_challenges_wallet      ON will_signing_challenges(wallet_address);

-- Will signatures table (Task 4 — stored signatures)
CREATE TABLE will_signatures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES will_documents(id) ON DELETE CASCADE,
    vault_id        VARCHAR(255) NOT NULL,
    wallet_address  VARCHAR(255) NOT NULL,
    document_hash   VARCHAR(64) NOT NULL,
    signature_hex   TEXT NOT NULL,
    signed_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_will_signatures_document_id ON will_signatures(document_id);
CREATE INDEX idx_will_signatures_wallet      ON will_signatures(wallet_address);
