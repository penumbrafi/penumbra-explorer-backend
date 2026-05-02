-- comprehensive DEX data tracking for market making
-- all transparent trading data: positions, swaps, liquidity, prices

-- track all assets seen on-chain
CREATE TABLE IF NOT EXISTS explorer_assets (
    asset_id TEXT PRIMARY KEY,
    decoded_passet TEXT NOT NULL,
    first_seen_height BIGINT NOT NULL,
    first_seen_time TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_explorer_assets_first_seen ON explorer_assets(first_seen_height);

-- track all liquidity positions (LP state)
CREATE TABLE IF NOT EXISTS dex_liquidity_positions (
    position_id TEXT PRIMARY KEY,
    decoded_position_id TEXT NOT NULL,
    trading_pair_asset1 TEXT NOT NULL,
    trading_pair_asset2 TEXT NOT NULL,
    reserves1_amount NUMERIC(39, 0) NOT NULL,
    reserves2_amount NUMERIC(39, 0) NOT NULL,
    state TEXT NOT NULL, -- 'opened', 'closed', 'withdrawn'
    fee_percentage NUMERIC(10, 2) NOT NULL,
    created_height BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_height BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (trading_pair_asset1) REFERENCES explorer_assets(asset_id),
    FOREIGN KEY (trading_pair_asset2) REFERENCES explorer_assets(asset_id)
);

CREATE INDEX idx_dex_positions_asset1 ON dex_liquidity_positions(trading_pair_asset1);
CREATE INDEX idx_dex_positions_asset2 ON dex_liquidity_positions(trading_pair_asset2);
CREATE INDEX idx_dex_positions_state ON dex_liquidity_positions(state);
CREATE INDEX idx_dex_positions_created ON dex_liquidity_positions(created_height);
CREATE INDEX idx_dex_positions_pair ON dex_liquidity_positions(trading_pair_asset1, trading_pair_asset2);

-- track batch swap executions (aggregated swaps per block)
CREATE TABLE IF NOT EXISTS dex_batch_swaps (
    id SERIAL PRIMARY KEY,
    block_height BIGINT NOT NULL,
    block_timestamp TIMESTAMPTZ NOT NULL,
    execution_type TEXT NOT NULL,
    total_input_amount NUMERIC(39, 0) NOT NULL,
    total_input_asset_id TEXT NOT NULL,
    total_output_amount NUMERIC(39, 0) NOT NULL,
    total_output_asset_id TEXT NOT NULL,
    individual_swaps_count INTEGER NOT NULL,
    raw_execution_data JSONB,
    FOREIGN KEY (total_input_asset_id) REFERENCES explorer_assets(asset_id),
    FOREIGN KEY (total_output_asset_id) REFERENCES explorer_assets(asset_id)
);

CREATE INDEX idx_dex_batch_swaps_height ON dex_batch_swaps(block_height);
CREATE INDEX idx_dex_batch_swaps_timestamp ON dex_batch_swaps(block_timestamp DESC);
CREATE INDEX idx_dex_batch_swaps_input_asset ON dex_batch_swaps(total_input_asset_id);
CREATE INDEX idx_dex_batch_swaps_output_asset ON dex_batch_swaps(total_output_asset_id);
CREATE INDEX idx_dex_batch_swaps_pair ON dex_batch_swaps(total_input_asset_id, total_output_asset_id);

-- track individual swaps within each batch
CREATE TABLE IF NOT EXISTS dex_individual_swaps (
    id SERIAL PRIMARY KEY,
    batch_swap_id INTEGER NOT NULL,
    swap_index INTEGER NOT NULL,
    input_amount NUMERIC(39, 0) NOT NULL,
    input_asset_id TEXT NOT NULL,
    output_amount NUMERIC(39, 0) NOT NULL,
    output_asset_id TEXT NOT NULL,
    route_steps_count INTEGER NOT NULL,
    FOREIGN KEY (batch_swap_id) REFERENCES dex_batch_swaps(id) ON DELETE CASCADE,
    FOREIGN KEY (input_asset_id) REFERENCES explorer_assets(asset_id),
    FOREIGN KEY (output_asset_id) REFERENCES explorer_assets(asset_id)
);

CREATE INDEX idx_dex_individual_swaps_batch ON dex_individual_swaps(batch_swap_id);
CREATE INDEX idx_dex_individual_swaps_input_asset ON dex_individual_swaps(input_asset_id);
CREATE INDEX idx_dex_individual_swaps_output_asset ON dex_individual_swaps(output_asset_id);

-- track routing steps for each swap (multi-hop routes)
CREATE TABLE IF NOT EXISTS dex_individual_swap_routes (
    id SERIAL PRIMARY KEY,
    individual_swap_id INTEGER NOT NULL,
    route_step INTEGER NOT NULL,
    amount NUMERIC(39, 0) NOT NULL,
    asset_id TEXT NOT NULL,
    FOREIGN KEY (individual_swap_id) REFERENCES dex_individual_swaps(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES explorer_assets(asset_id)
);

CREATE INDEX idx_dex_swap_routes_swap ON dex_individual_swap_routes(individual_swap_id);
CREATE INDEX idx_dex_swap_routes_asset ON dex_individual_swap_routes(asset_id);

-- view: active liquidity by trading pair
CREATE VIEW active_liquidity_by_pair AS
SELECT
    trading_pair_asset1,
    trading_pair_asset2,
    COUNT(*) as active_positions,
    SUM(reserves1_amount) as total_reserves1,
    SUM(reserves2_amount) as total_reserves2,
    AVG(fee_percentage) as avg_fee_percentage
FROM dex_liquidity_positions
WHERE state = 'opened'
GROUP BY trading_pair_asset1, trading_pair_asset2;

-- view: trading volume by asset (24h)
CREATE VIEW trading_volume_24h AS
SELECT
    total_input_asset_id as asset_id,
    SUM(total_input_amount) as volume_24h,
    COUNT(*) as swap_count_24h,
    MIN(block_timestamp) as period_start,
    MAX(block_timestamp) as period_end
FROM dex_batch_swaps
WHERE block_timestamp > NOW() - INTERVAL '24 hours'
GROUP BY total_input_asset_id

UNION ALL

SELECT
    total_output_asset_id as asset_id,
    SUM(total_output_amount) as volume_24h,
    COUNT(*) as swap_count_24h,
    MIN(block_timestamp) as period_start,
    MAX(block_timestamp) as period_end
FROM dex_batch_swaps
WHERE block_timestamp > NOW() - INTERVAL '24 hours'
GROUP BY total_output_asset_id;

-- view: price discovery from recent swaps
CREATE VIEW recent_swap_prices AS
SELECT
    total_input_asset_id,
    total_output_asset_id,
    AVG(CAST(total_output_amount AS NUMERIC) / CAST(total_input_amount AS NUMERIC)) as avg_price,
    COUNT(*) as swap_count,
    MAX(block_timestamp) as latest_swap
FROM dex_batch_swaps
WHERE block_timestamp > NOW() - INTERVAL '1 hour'
  AND total_input_amount > 0
GROUP BY total_input_asset_id, total_output_asset_id;
