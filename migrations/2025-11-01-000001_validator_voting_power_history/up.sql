-- Create table to store historical voting power changes for validators
CREATE TABLE IF NOT EXISTS validator_voting_power_history (
    id BIGSERIAL PRIMARY KEY,
    validator_identity_key TEXT NOT NULL,
    voting_power BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    epoch_index BIGINT,

    -- Unique constraint to prevent duplicate entries
    UNIQUE(validator_identity_key, block_height)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_validator_voting_power_history_validator
    ON validator_voting_power_history(validator_identity_key);

CREATE INDEX IF NOT EXISTS idx_validator_voting_power_history_timestamp
    ON validator_voting_power_history(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_validator_voting_power_history_block_height
    ON validator_voting_power_history(block_height DESC);

-- Composite index for common query pattern (validator + time range)
CREATE INDEX IF NOT EXISTS idx_validator_voting_power_history_validator_timestamp
    ON validator_voting_power_history(validator_identity_key, timestamp DESC);
