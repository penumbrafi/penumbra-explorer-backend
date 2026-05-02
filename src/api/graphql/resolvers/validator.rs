use crate::api::graphql::types::validator::{ValidatorDetails, ValidatorHomepageData, VotingPowerHistoryEntry};
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
            "#
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
            "#
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
            "#
        )
        .bind(&validator_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(entries)
}
