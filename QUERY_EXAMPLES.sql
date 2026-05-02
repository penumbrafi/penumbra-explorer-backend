-- penumbra trading intelligence: example queries

-- === IBC CAPITAL FLOWS ===

-- monitor large withdrawals (potential sell pressure)
SELECT 
    encode(tx_hash, 'hex') as tx,
    asset_id,
    amount::numeric / 1e6 as amount_formatted,
    destination_chain_address,
    timestamp
FROM recent_large_withdrawals
ORDER BY timestamp DESC
LIMIT 20;

-- net ibc flows by asset (capital in vs out)
SELECT 
    asset_id_hex,
    total_inbound::numeric / 1e6 as inbound,
    total_outbound::numeric / 1e6 as outbound,
    net_flow::numeric / 1e6 as net,
    transfer_count
FROM ibc_net_flows
ORDER BY ABS(net_flow) DESC;

-- ibc flow velocity last 24h
SELECT 
    channel_id,
    direction,
    transfer_count,
    total_volume::numeric / 1e6 as volume,
    avg_transfer_size::numeric / 1e6 as avg_size,
    period_start,
    period_end
FROM ibc_flow_velocity_24h
ORDER BY total_volume DESC;

-- === DEX TRADING DATA ===

-- active liquidity by trading pair
SELECT 
    trading_pair_asset1,
    trading_pair_asset2,
    active_positions,
    total_reserves1::numeric / 1e6 as reserves1,
    total_reserves2::numeric / 1e6 as reserves2,
    avg_fee_percentage
FROM active_liquidity_by_pair
ORDER BY active_positions DESC;

-- recent swap prices (price discovery)
SELECT 
    total_input_asset_id as from_asset,
    total_output_asset_id as to_asset,
    avg_price,
    swap_count,
    latest_swap
FROM recent_swap_prices
ORDER BY latest_swap DESC;

-- 24h trading volume by asset
SELECT 
    asset_id,
    SUM(volume_24h)::numeric / 1e6 as total_volume,
    SUM(swap_count_24h) as total_swaps
FROM trading_volume_24h
GROUP BY asset_id
ORDER BY total_volume DESC;

-- multi-hop swap routes (arbitrage analysis)
SELECT 
    bs.block_height,
    bs.block_timestamp,
    is_swap.input_asset_id,
    is_swap.output_asset_id,
    is_swap.route_steps_count,
    isr.route_step,
    isr.asset_id as intermediate_asset,
    isr.amount::numeric / 1e6 as amount
FROM dex_batch_swaps bs
JOIN dex_individual_swaps is_swap ON bs.id = is_swap.batch_swap_id
JOIN dex_individual_swap_routes isr ON is_swap.id = isr.individual_swap_id
WHERE is_swap.route_steps_count > 2
ORDER BY bs.block_height DESC, isr.route_step
LIMIT 50;

-- === GOVERNANCE IMPACT ===

-- active governance proposals
SELECT 
    proposal_id,
    title,
    state,
    start_block_height,
    end_block_height,
    end_timestamp
FROM active_governance_proposals
ORDER BY end_block_height;

-- proposal voting summary
SELECT 
    proposal_id,
    title,
    state,
    total_votes,
    yes_power::numeric / 1e6 as yes_voting_power,
    no_power::numeric / 1e6 as no_voting_power,
    abstain_power::numeric / 1e6 as abstain_power,
    total_voting_power::numeric / 1e6 as total_power
FROM proposal_voting_summary
ORDER BY proposal_id DESC;

-- current chain parameters
SELECT 
    chain_id,
    deposit_amount::numeric / 1e6 as proposal_deposit,
    passing_threshold,
    slashing_threshold,
    valid_quorum,
    proposal_voting_blocks,
    updated_height,
    updated_at
FROM governance_parameters;

-- === COMBINED ANALYSIS ===

-- correlation: large ibc outflows before price drops
WITH outflow_spikes AS (
    SELECT 
        DATE_TRUNC('hour', timestamp) as hour,
        asset_id,
        COUNT(*) as withdrawal_count,
        SUM(amount) as total_outflow
    FROM ibc_withdrawals
    WHERE timestamp > NOW() - INTERVAL '7 days'
    GROUP BY hour, asset_id
    HAVING COUNT(*) > 5
),
price_changes AS (
    SELECT 
        DATE_TRUNC('hour', block_timestamp) as hour,
        total_input_asset_id as asset_id,
        AVG(CAST(total_output_amount AS NUMERIC) / CAST(total_input_amount AS NUMERIC)) as avg_price
    FROM dex_batch_swaps
    WHERE block_timestamp > NOW() - INTERVAL '7 days'
    GROUP BY hour, asset_id
)
SELECT 
    o.hour,
    o.asset_id,
    o.withdrawal_count,
    o.total_outflow::numeric / 1e6 as outflow,
    p.avg_price,
    LAG(p.avg_price) OVER (PARTITION BY o.asset_id ORDER BY o.hour) as prev_price,
    (p.avg_price - LAG(p.avg_price) OVER (PARTITION BY o.asset_id ORDER BY o.hour)) / 
        NULLIF(LAG(p.avg_price) OVER (PARTITION BY o.asset_id ORDER BY o.hour), 0) * 100 as price_change_pct
FROM outflow_spikes o
LEFT JOIN price_changes p ON o.hour = p.hour AND o.asset_id = p.asset_id
ORDER BY o.hour DESC, o.total_outflow DESC;

-- liquidity depth vs trading volume ratio
SELECT 
    lp.trading_pair_asset1,
    lp.trading_pair_asset2,
    lp.total_reserves1::numeric / 1e6 as reserves1,
    lp.total_reserves2::numeric / 1e6 as reserves2,
    COALESCE(SUM(vol.volume_24h), 0)::numeric / 1e6 as volume_24h,
    CASE 
        WHEN SUM(vol.volume_24h) > 0 THEN 
            (lp.total_reserves1 / SUM(vol.volume_24h))::numeric 
        ELSE NULL 
    END as liquidity_to_volume_ratio
FROM active_liquidity_by_pair lp
LEFT JOIN trading_volume_24h vol ON lp.trading_pair_asset1 = vol.asset_id
GROUP BY lp.trading_pair_asset1, lp.trading_pair_asset2, lp.total_reserves1, lp.total_reserves2
ORDER BY liquidity_to_volume_ratio DESC NULLS LAST;
