-- Drop the view first
DROP VIEW IF EXISTS validator_voting_power_history_with_name;

-- Drop indexes
DROP INDEX IF EXISTS idx_validator_voting_power_history_validator_timestamp;
DROP INDEX IF EXISTS idx_validator_voting_power_history_block_height;
DROP INDEX IF EXISTS idx_validator_voting_power_history_timestamp;
DROP INDEX IF EXISTS idx_validator_voting_power_history_validator;

-- Drop the table
DROP TABLE IF EXISTS validator_voting_power_history;
