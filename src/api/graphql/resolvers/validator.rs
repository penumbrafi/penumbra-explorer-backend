use crate::api::graphql::types::validator::{
    GrowthWindow, ValidatorDetails, ValidatorGrowthEntry, ValidatorHomepageData,
    VotingPowerHistoryEntry,
};
use crate::api::graphql::types::ValidatorFilter;
use async_graphql::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Resolves validators homepage query
///
/// # Errors
///
/// Returns an error if database queries fail
pub async fn resolve_validators_homepage(
    ctx: &Context<'_>,
    filter: Option<ValidatorFilter>,
) -> async_graphql::Result<ValidatorHomepageData> {
    ValidatorHomepageData::fetch_homepage_data(ctx, filter).await
}

/// Resolves validator details query
///
/// # Errors
///
/// Returns an error if database queries fail
pub async fn resolve_validator_details(
    ctx: &Context<'_>,
    id: String,
) -> async_graphql::Result<Option<ValidatorDetails>> {
    let pool = ctx.data::<PgPool>()?;
    ValidatorDetails::get_by_address(pool, &id).await
}

/// Resolves validator voting power history query
///
/// # Errors
///
/// Returns an error if database queries fail
pub async fn resolve_validator_voting_power_history(
    ctx: &Context<'_>,
    validator_id: String,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: Option<i32>,
) -> async_graphql::Result<Vec<VotingPowerHistoryEntry>> {
    let pool = ctx.data::<PgPool>()?;

    let limit = limit.unwrap_or(1000).min(10000);

    let entries = if let (Some(start), Some(end)) = (start_time, end_time) {
        sqlx::query_as::<_, VotingPowerHistoryEntry>(
            r#"
            SELECT
                validator_identity_key,
                voting_power,
                block_height,
                timestamp
            FROM validator_voting_power_history
            WHERE validator_identity_key = $1
                AND timestamp >= $2
                AND timestamp <= $3
            ORDER BY timestamp ASC
            LIMIT $4
            "#,
        )
        .bind(&validator_id)
        .bind(start)
        .bind(end)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else if let Some(start) = start_time {
        sqlx::query_as::<_, VotingPowerHistoryEntry>(
            r#"
            SELECT
                validator_identity_key,
                voting_power,
                block_height,
                timestamp
            FROM validator_voting_power_history
            WHERE validator_identity_key = $1
                AND timestamp >= $2
            ORDER BY timestamp ASC
            LIMIT $3
            "#,
        )
        .bind(&validator_id)
        .bind(start)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, VotingPowerHistoryEntry>(
            r#"
            SELECT
                validator_identity_key,
                voting_power,
                block_height,
                timestamp
            FROM validator_voting_power_history
            WHERE validator_identity_key = $1
            ORDER BY timestamp ASC
            LIMIT $2
            "#,
        )
        .bind(&validator_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(entries)
}

/// Resolves validators_by_growth: ranks validators by voting-power growth
/// over the requested time window. Powers the "fastest-growing operators"
/// surface that incentivizes new delegations to active validators.
///
/// Implementation: single SQL with one CTE per window — joins the latest
/// snapshot per validator against the snapshot closest to (now - window).
/// Validators without a historical snapshot are returned with past=0 so
/// the client can mark them "new". O(N validators) regardless of N or
/// window size; no per-validator round trip.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn resolve_validators_by_growth(
    ctx: &Context<'_>,
    window: GrowthWindow,
    limit: Option<i32>,
) -> async_graphql::Result<Vec<ValidatorGrowthEntry>> {
    let pool = ctx.data::<PgPool>()?;
    let limit = limit.unwrap_or(50).clamp(1, 500);

    let interval = window.interval_sql();
    let sql = format!(
        r#"
        WITH latest AS (
            SELECT DISTINCT ON (validator_identity_key)
                validator_identity_key,
                voting_power,
                timestamp
            FROM validator_voting_power_history
            ORDER BY validator_identity_key, timestamp DESC
        ),
        past AS (
            SELECT DISTINCT ON (validator_identity_key)
                validator_identity_key,
                voting_power
            FROM validator_voting_power_history
            WHERE timestamp <= NOW() - INTERVAL '{interval}'
            ORDER BY validator_identity_key, timestamp DESC
        )
        SELECT
            l.validator_identity_key            AS validator_identity_key,
            v.name                              AS name,
            v.state                             AS state,
            l.voting_power                      AS current_voting_power,
            COALESCE(p.voting_power, 0)         AS past_voting_power,
            l.voting_power - COALESCE(p.voting_power, 0) AS growth_absolute,
            CASE
                WHEN COALESCE(p.voting_power, 0) > 0
                    THEN ((l.voting_power - p.voting_power)::float8
                          / p.voting_power::float8) * 100.0
                ELSE NULL
            END                                 AS growth_pct
        FROM latest l
        LEFT JOIN past p USING (validator_identity_key)
        LEFT JOIN validators v ON v.identity_key = l.validator_identity_key
        WHERE l.voting_power > 0
        ORDER BY growth_pct DESC NULLS LAST, l.voting_power DESC
        LIMIT $1
        "#
    );

    let entries = sqlx::query_as::<_, ValidatorGrowthEntry>(&sql)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    Ok(entries)
}
