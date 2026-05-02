DROP VIEW IF EXISTS recent_large_withdrawals;
DROP VIEW IF EXISTS ibc_outflows_by_asset;
DROP INDEX IF EXISTS idx_ibc_withdrawals_destination;
DROP INDEX IF EXISTS idx_ibc_withdrawals_timestamp;
DROP INDEX IF EXISTS idx_ibc_withdrawals_block_height;
DROP INDEX IF EXISTS idx_ibc_withdrawals_denom;
DROP INDEX IF EXISTS idx_ibc_withdrawals_asset;
DROP TABLE IF EXISTS ibc_withdrawals;
