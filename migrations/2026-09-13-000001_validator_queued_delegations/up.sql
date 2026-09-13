-- Expose pindexer's per-block queued delegation/undelegation flow on
-- validator_performance. queued_* accumulate between epoch boundaries and
-- reset when voting_power is applied, so they complement (not replace) it.
DROP VIEW IF EXISTS validator_performance;

CREATE OR REPLACE VIEW validator_performance AS
WITH commission_rates AS (
    SELECT
        identity_key,
        ROUND(SUM(rate_bps)::numeric / 100.0, 2) as commission_rate_percentage
    FROM validator_funding_streams
    GROUP BY identity_key
)
SELECT
    v.identity_key,
    v.decoded_address,
    COALESCE(v.name, '') as name,
    v.website,
    v.description,
    v.state,
    COALESCE(v.bonding_state, '') as bonding_state,
    v.voting_power,
    v.voting_power_active_percentage,
    v.first_seen_height,
    v.first_seen_time,
    COALESCE(cr.commission_rate_percentage, 0.0) as commission_rate,
    COALESCE(us.missed_blocks, 0) as missed_blocks,
    COALESCE(us.signed_blocks, 0) as signed_blocks,
    COALESCE(us.total_blocks, 0) as total_tracked_blocks,
    COALESCE(us.uptime_percentage, 0.00) as uptime_percentage,
    COALESCE(svs.queued_delegations, 0) as queued_delegations,
    COALESCE(svs.queued_undelegations, 0) as queued_undelegations
FROM
    validators v
LEFT JOIN validator_uptime_stats us ON v.identity_key = us.identity_key
LEFT JOIN commission_rates cr ON v.identity_key = cr.identity_key
LEFT JOIN stake_validator_set svs ON svs.ik = v.identity_key
ORDER BY
    v.voting_power DESC;
