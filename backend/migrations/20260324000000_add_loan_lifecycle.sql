-- ──────────────────────────────────────────────────────────────────────────────
-- Loan Lifecycle Tracker
-- Tracks the state of every loan through: Active → Repaid | Overdue | Liquidated
-- ──────────────────────────────────────────────────────────────────────────────

CREATE TYPE loan_lifecycle_status AS ENUM (
    'active',
    'repaid',
    'overdue',
    'liquidated'
);

CREATE TABLE loan_lifecycle (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Owning user & optional plan context
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id             UUID REFERENCES plans(id) ON DELETE SET NULL,

    -- Asset details
    borrow_asset        VARCHAR(20)  NOT NULL,
    collateral_asset    VARCHAR(20)  NOT NULL,

    -- Amounts (stored as NUMERIC for precision)
    principal           NUMERIC(30, 8) NOT NULL CHECK (principal > 0),
    interest_rate_bps   INTEGER       NOT NULL CHECK (interest_rate_bps >= 0),
    collateral_amount   NUMERIC(30, 8) NOT NULL CHECK (collateral_amount >= 0),
    amount_repaid       NUMERIC(30, 8) NOT NULL DEFAULT 0 CHECK (amount_repaid >= 0),

    -- Status tracking
    status              loan_lifecycle_status NOT NULL DEFAULT 'active',
    due_date            TIMESTAMP WITH TIME ZONE NOT NULL,

    -- Optional on-chain reference
    transaction_hash    VARCHAR(255),

    -- Timestamps
    created_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    repaid_at           TIMESTAMP WITH TIME ZONE,
    liquidated_at       TIMESTAMP WITH TIME ZONE
);

-- ── Indexes ──────────────────────────────────────────────────────────────────
CREATE INDEX idx_loan_lifecycle_user_id   ON loan_lifecycle(user_id);
CREATE INDEX idx_loan_lifecycle_plan_id   ON loan_lifecycle(plan_id);
CREATE INDEX idx_loan_lifecycle_status    ON loan_lifecycle(status);
CREATE INDEX idx_loan_lifecycle_due_date  ON loan_lifecycle(due_date);
CREATE INDEX idx_loan_lifecycle_created   ON loan_lifecycle(created_at DESC);

-- ── Auto-update updated_at via trigger ───────────────────────────────────────
CREATE OR REPLACE FUNCTION update_loan_lifecycle_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_loan_lifecycle_updated_at
BEFORE UPDATE ON loan_lifecycle
FOR EACH ROW EXECUTE FUNCTION update_loan_lifecycle_updated_at();
