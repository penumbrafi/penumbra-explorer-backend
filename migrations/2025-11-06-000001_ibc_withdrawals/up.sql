-- track ibc withdrawals (ics20) to monitor capital outflows for trading signals
CREATE TABLE IF NOT EXISTS ibc_withdrawals (
    id BIGSERIAL PRIMARY KEY,
    tx_hash BYTEA NOT NULL,
    asset_id TEXT NOT NULL,
    amount NUMERIC(39, 0) NOT NULL,
    denom TEXT NOT NULL,
    destination_chain_address TEXT NOT NULL,
    source_channel TEXT,
    return_address TEXT,
    timeout_height BIGINT,
    timeout_timestamp BIGINT,
    use_compat_address BOOLEAN DEFAULT FALSE,
    block_height BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    UNIQUE(tx_hash, asset_id, destination_chain_address)
);

CREATE INDEX idx_ibc_withdrawals_asset ON ibc_withdrawals(asset_id);
CREATE INDEX idx_ibc_withdrawals_denom ON ibc_withdrawals(denom);
CREATE INDEX idx_ibc_withdrawals_block_height ON ibc_withdrawals(block_height);
CREATE INDEX idx_ibc_withdrawals_timestamp ON ibc_withdrawals(timestamp DESC);
CREATE INDEX idx_ibc_withdrawals_destination ON ibc_withdrawals(destination_chain_address);

-- view for aggregated outflows by asset (detect sell pressure)
CREATE VIEW ibc_outflows_by_asset AS
SELECT
    asset_id,
    denom,
    SUM(amount) as total_withdrawn,
    COUNT(*) as withdrawal_count,
    MIN(timestamp) as first_withdrawal,
    MAX(timestamp) as latest_withdrawal
FROM ibc_withdrawals
GROUP BY asset_id, denom;

-- view for recent large withdrawals (whale watching)
CREATE VIEW recent_large_withdrawals AS
SELECT
    tx_hash,
    asset_id,
    denom,
    amount,
    destination_chain_address,
    block_height,
    timestamp
FROM ibc_withdrawals
WHERE timestamp > NOW() - INTERVAL '7 days'
ORDER BY amount DESC;
