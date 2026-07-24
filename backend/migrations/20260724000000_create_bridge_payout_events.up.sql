-- Track every BridgePay event captured from the Soroban contract so the
-- listener can skip events it has already processed (idempotency) and give
-- the webhook dispatcher a durable record for audit / replay.
CREATE TABLE IF NOT EXISTS bridge_payout_events (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Coordinates that uniquely identify a single contract event on-chain.
    -- The combination (contract_id, ledger_sequence, tx_hash, event_index)
    -- is used as the natural dedup key.
    contract_id         TEXT        NOT NULL,
    ledger_sequence     BIGINT      NOT NULL,
    tx_hash             TEXT        NOT NULL,
    event_index         INTEGER     NOT NULL DEFAULT 0,

    -- Decoded BridgePayoutEvent fields (mirrors the Soroban struct).
    owner_address       TEXT        NOT NULL,
    token_address       TEXT        NOT NULL,
    beneficiary_address TEXT        NOT NULL,
    destination_chain   TEXT        NOT NULL,
    destination_address TEXT        NOT NULL,
    gross_amount        BIGINT      NOT NULL,
    fee_amount          BIGINT      NOT NULL DEFAULT 0,
    net_amount          BIGINT      NOT NULL,
    source_chain        TEXT        NOT NULL DEFAULT '',
    source_tx_hash      TEXT        NOT NULL DEFAULT '',

    -- Processing state.
    raw_event           JSONB       NOT NULL DEFAULT '{}',
    processed_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Fast duplicate-check: the listener performs INSERT … ON CONFLICT DO NOTHING
-- using this unique constraint to guarantee exactly-once persistence.
CREATE UNIQUE INDEX IF NOT EXISTS uidx_bridge_payout_events_dedup
    ON bridge_payout_events (contract_id, ledger_sequence, tx_hash, event_index);

-- Support efficient pagination when replaying recent events.
CREATE INDEX IF NOT EXISTS idx_bridge_payout_events_ledger
    ON bridge_payout_events (ledger_sequence DESC);
