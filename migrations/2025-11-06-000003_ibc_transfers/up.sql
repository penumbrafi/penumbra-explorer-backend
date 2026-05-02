-- complete IBC transfer tracking (both directions) from packet events
-- this complements ibc_withdrawals (from transaction actions)

CREATE TABLE IF NOT EXISTS ibc_transfers (
    id BIGSERIAL PRIMARY KEY,
    client_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('inbound', 'outbound', 'other')),
    amount BIGINT NOT NULL,
    asset_id BYTEA,
    timestamp TIMESTAMPTZ NOT NULL,
    tx_hash BYTEA,
    status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'expired', 'error'))
);

CREATE INDEX idx_ibc_transfers_client ON ibc_transfers(client_id);
CREATE INDEX idx_ibc_transfers_channel ON ibc_transfers(channel_id);
CREATE INDEX idx_ibc_transfers_direction ON ibc_transfers(direction);
CREATE INDEX idx_ibc_transfers_timestamp ON ibc_transfers(timestamp DESC);
CREATE INDEX idx_ibc_transfers_status ON ibc_transfers(status);
CREATE INDEX idx_ibc_transfers_asset ON ibc_transfers(asset_id) WHERE asset_id IS NOT NULL;

-- view: net ibc flows by asset
CREATE VIEW ibc_net_flows AS
SELECT
    encode(asset_id, 'hex') as asset_id_hex,
    SUM(CASE WHEN direction = 'inbound' THEN amount ELSE 0 END) as total_inbound,
    SUM(CASE WHEN direction = 'outbound' THEN amount ELSE 0 END) as total_outbound,
    SUM(CASE WHEN direction = 'inbound' THEN amount ELSE -amount END) as net_flow,
    COUNT(*) as transfer_count
FROM ibc_transfers
WHERE asset_id IS NOT NULL
GROUP BY asset_id;

-- view: ibc flow velocity (last 24h)
CREATE VIEW ibc_flow_velocity_24h AS
SELECT
    channel_id,
    direction,
    COUNT(*) as transfer_count,
    SUM(amount) as total_volume,
    AVG(amount) as avg_transfer_size,
    MIN(timestamp) as period_start,
    MAX(timestamp) as period_end
FROM ibc_transfers
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY channel_id, direction;
