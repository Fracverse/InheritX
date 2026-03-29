-- Witness verification system for will documents (Issue #331)

CREATE TABLE will_witnesses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES will_documents(id) ON DELETE CASCADE,
    inviter_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_address  VARCHAR(255),
    email           VARCHAR(255),
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    signature_hex   TEXT,
    signed_at       TIMESTAMP WITH TIME ZONE,
    invited_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT witness_contact CHECK (wallet_address IS NOT NULL OR email IS NOT NULL)
);

CREATE INDEX idx_will_witnesses_document_id ON will_witnesses(document_id);
CREATE INDEX idx_will_witnesses_status ON will_witnesses(status);
