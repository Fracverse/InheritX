-- Issue #256: Data Warehouse & Metrics Aggregation
-- Stores periodic snapshots of lending pool performance for long-term analysis.
CREATE TABLE IF NOT EXISTS lending_metrics_snapshots (
    id                 BIGSERIAL PRIMARY KEY,
    snapshot_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tvl                DOUBLE PRECISION NOT NULL,
    total_borrowed     DOUBLE PRECISION NOT NULL,
    utilization_rate   DOUBLE PRECISION NOT NULL,
    active_loans_count BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lending_metrics_snapshots_at
    ON lending_metrics_snapshots (snapshot_at DESC);
