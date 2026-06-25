-- Assumptions:
-- - UUID primary keys for distributed-friendly identifiers.
-- - Wallet/address columns are TEXT (Stellar-style addresses, no DB-level format validation).
-- - Token amounts use NUMERIC(78, 0) for large on-chain integer semantics.
-- - grace_period and last_ping store unix epoch seconds as BIGINT.
-- - plans.owner_address is not FK-linked to users (owners may exist before KYC).
--
-- Index rationale:
-- - plans(owner_address): list plans by owner (GET /api/plans?owner=).
-- - beneficiaries(plan_id): load beneficiaries for a plan.
-- - payouts(plan_id): list payouts for a plan.
-- - payouts(beneficiary_address): anchor status lookup by beneficiary.
--
-- Constraint smoke tests (must fail):
-- - allocation_bps = 10001 on one beneficiary -> CHECK violation.
-- - Two beneficiaries 6000 + 5000 bps on same plan -> trigger violation.
-- - payout_type = 'wire' -> invalid enum.
-- - beneficiaries.plan_id referencing missing plan -> FK violation.
-- - Duplicate users.wallet_address -> UNIQUE violation.

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    kyc_status kyc_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_wallet_address_unique UNIQUE (wallet_address)
);

CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_address TEXT NOT NULL,
    token_address TEXT NOT NULL,
    amount NUMERIC(78, 0) NOT NULL CHECK (amount >= 0),
    grace_period BIGINT NOT NULL,
    earn_yield BOOLEAN NOT NULL DEFAULT FALSE,
    last_ping BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX plans_owner_address_idx ON plans (owner_address);

CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    allocation_bps INTEGER NOT NULL
        CHECK (allocation_bps >= 0 AND allocation_bps <= 10000),
    fiat_anchor_info TEXT NOT NULL DEFAULT '',
    CONSTRAINT beneficiaries_plan_wallet_unique UNIQUE (plan_id, wallet_address)
);

CREATE INDEX beneficiaries_plan_id_idx ON beneficiaries (plan_id);

CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE RESTRICT,
    beneficiary_address TEXT NOT NULL,
    amount NUMERIC(78, 0) NOT NULL CHECK (amount > 0),
    payout_type payout_type NOT NULL,
    status payout_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX payouts_plan_id_idx ON payouts (plan_id);
CREATE INDEX payouts_beneficiary_address_idx ON payouts (beneficiary_address);

CREATE OR REPLACE FUNCTION check_plan_allocation_total()
RETURNS TRIGGER AS $$
DECLARE
    target_plan_id UUID;
    total_bps INTEGER;
BEGIN
    target_plan_id := COALESCE(NEW.plan_id, OLD.plan_id);

    SELECT COALESCE(SUM(allocation_bps), 0)
    INTO total_bps
    FROM beneficiaries
    WHERE plan_id = target_plan_id;

    IF total_bps > 10000 THEN
        RAISE EXCEPTION 'Total allocation_bps for plan % exceeds 10000', target_plan_id;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER beneficiaries_allocation_total_check
    AFTER INSERT OR UPDATE OR DELETE ON beneficiaries
    FOR EACH ROW
    EXECUTE FUNCTION check_plan_allocation_total();
