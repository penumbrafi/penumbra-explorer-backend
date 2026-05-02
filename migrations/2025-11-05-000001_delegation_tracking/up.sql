-- track delegation actions for market making and liquidity analysis
CREATE TABLE IF NOT EXISTS delegates (
    id BIGSERIAL PRIMARY KEY,
    tx_hash BYTEA NOT NULL,
    validator_identity_key TEXT NOT NULL,
    delegation_amount NUMERIC(39, 0) NOT NULL,
    unbonded_amount NUMERIC(39, 0) NOT NULL,
    epoch_index BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,

    UNIQUE(tx_hash, validator_identity_key)
);

CREATE INDEX IF NOT EXISTS idx_delegates_validator
    ON delegates(validator_identity_key);

CREATE INDEX IF NOT EXISTS idx_delegates_timestamp
    ON delegates(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_delegates_block_height
    ON delegates(block_height DESC);

CREATE INDEX IF NOT EXISTS idx_delegates_validator_timestamp
    ON delegates(validator_identity_key, timestamp DESC);

-- track undelegation actions to monitor sell pressure
CREATE TABLE IF NOT EXISTS undelegates (
    id BIGSERIAL PRIMARY KEY,
    tx_hash BYTEA NOT NULL,
    validator_identity_key TEXT NOT NULL,
    delegation_amount NUMERIC(39, 0) NOT NULL,
    unbonded_amount NUMERIC(39, 0) NOT NULL,
    epoch_index BIGINT NOT NULL,
    unbonding_start_height BIGINT NOT NULL,
    release_height BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    claimed BOOLEAN DEFAULT FALSE,

    UNIQUE(tx_hash, validator_identity_key, unbonding_start_height)
);

CREATE INDEX IF NOT EXISTS idx_undelegates_validator
    ON undelegates(validator_identity_key);

CREATE INDEX IF NOT EXISTS idx_undelegates_timestamp
    ON undelegates(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_undelegates_block_height
    ON undelegates(block_height DESC);

CREATE INDEX IF NOT EXISTS idx_undelegates_release_height
    ON undelegates(release_height);

CREATE INDEX IF NOT EXISTS idx_undelegates_validator_timestamp
    ON undelegates(validator_identity_key, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_undelegates_pending
    ON undelegates(release_height, validator_identity_key) WHERE claimed = FALSE;

-- view for pending undelegations by validator
CREATE VIEW pending_undelegations AS
SELECT
    validator_identity_key,
    SUM(unbonded_amount) as total_unbonding,
    COUNT(*) as undelegate_count,
    MIN(release_height) as next_release_height
FROM undelegates
WHERE claimed = FALSE
GROUP BY validator_identity_key;
