-- Issue #1270: require an auditable record for every lending deposit and
-- withdrawal the backend relays, so a wallet's deposit history and derived
-- earnings can be returned next to the contract's authoritative share balance.
CREATE TABLE lending_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    asset TEXT NOT NULL,
    tx_type TEXT NOT NULL CHECK (tx_type IN ('DEPOSIT', 'WITHDRAW')),
    -- Underlying token amount, in stroop-equivalent base units.
    amount NUMERIC(38, 0) NOT NULL DEFAULT 0,
    -- Pool shares minted (deposit) or burned (withdrawal).
    shares NUMERIC(38, 0) NOT NULL DEFAULT 0,
    tx_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX lending_transactions_wallet_created_at_idx
    ON lending_transactions (wallet_address, created_at DESC);

-- Issue #1271: recommendations are persisted so a later accept/reject decision
-- targets the exact allocation set the user was shown.
CREATE TABLE ai_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    owner_address TEXT NOT NULL,
    -- The full OptimizationRecommendation DTO as returned to the client.
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'rejected')),
    response_reason TEXT,
    responded_by TEXT,
    responded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ai_recommendations_plan_status_idx
    ON ai_recommendations (plan_id, status);
