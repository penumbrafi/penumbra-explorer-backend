use crate::api::graphql::types::{Delegate, Undelegate, ValidatorStakingStats};
use async_graphql::Context;
use sqlx::PgPool;

pub async fn resolve_validator_delegates(
    ctx: &Context<'_>,
    validator_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
) -> async_graphql::Result<Vec<Delegate>> {
    let db = ctx.data::<PgPool>()?;
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    let delegates = sqlx::query_as::<_, Delegate>(
        r"
        SELECT id, tx_hash, validator_identity_key,
               delegation_amount::TEXT as delegation_amount,
               unbonded_amount::TEXT as unbonded_amount,
               epoch_index, block_height, timestamp
        FROM delegates
        WHERE validator_identity_key = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        ",
    )
    .bind(&validator_id)
    .bind(i64::from(limit))
    .bind(i64::from(offset))
    .fetch_all(db)
    .await?;

    Ok(delegates)
}

pub async fn resolve_validator_undelegates(
    ctx: &Context<'_>,
    validator_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
    pending_only: Option<bool>,
) -> async_graphql::Result<Vec<Undelegate>> {
    let db = ctx.data::<PgPool>()?;
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);
    let pending_only = pending_only.unwrap_or(false);

    let query = if pending_only {
        r"
        SELECT id, tx_hash, validator_identity_key,
               delegation_amount::TEXT as delegation_amount,
               unbonded_amount::TEXT as unbonded_amount,
               epoch_index, unbonding_start_height, release_height,
               block_height, timestamp, claimed
        FROM undelegates
        WHERE validator_identity_key = $1
        AND claimed = FALSE
        ORDER BY release_height ASC
        LIMIT $2 OFFSET $3
        "
    } else {
        r"
        SELECT id, tx_hash, validator_identity_key,
               delegation_amount::TEXT as delegation_amount,
               unbonded_amount::TEXT as unbonded_amount,
               epoch_index, unbonding_start_height, release_height,
               block_height, timestamp, claimed
        FROM undelegates
        WHERE validator_identity_key = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "
    };

    let undelegates = sqlx::query_as::<_, Undelegate>(query)
        .bind(&validator_id)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(db)
        .await?;

    Ok(undelegates)
}

pub async fn resolve_validator_staking_stats(
    ctx: &Context<'_>,
    validator_id: String,
) -> async_graphql::Result<Option<ValidatorStakingStats>> {
    let db = ctx.data::<PgPool>()?;

    let total_delegations = sqlx::query_scalar::<_, Option<String>>(
        r"
        SELECT COALESCE(SUM(unbonded_amount), 0)::TEXT
        FROM delegates
        WHERE validator_identity_key = $1
        ",
    )
    .bind(&validator_id)
    .fetch_one(db)
    .await?
    .unwrap_or_else(|| "0".to_string());

    let total_undelegations = sqlx::query_scalar::<_, Option<String>>(
        r"
        SELECT COALESCE(SUM(unbonded_amount), 0)::TEXT
        FROM undelegates
        WHERE validator_identity_key = $1
        ",
    )
    .bind(&validator_id)
    .fetch_one(db)
    .await?
    .unwrap_or_else(|| "0".to_string());

    let pending_stats: Option<(String, i64, Option<i64>)> = sqlx::query_as(
        r"
        SELECT
            COALESCE(SUM(unbonded_amount), 0)::TEXT,
            COUNT(*)::BIGINT,
            MIN(release_height)
        FROM undelegates
        WHERE validator_identity_key = $1
        AND claimed = FALSE
        ",
    )
    .bind(&validator_id)
    .fetch_optional(db)
    .await?;

    let (pending_undelegations, pending_undelegate_count, next_release_height) =
        pending_stats.unwrap_or_else(|| ("0".to_string(), 0, None));

    Ok(Some(ValidatorStakingStats {
        validator_identity_key: validator_id,
        total_delegations,
        total_undelegations,
        pending_undelegations,
        pending_undelegate_count,
        next_release_height,
    }))
}

pub async fn resolve_pending_undelegations(
    ctx: &Context<'_>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> async_graphql::Result<Vec<Undelegate>> {
    let db = ctx.data::<PgPool>()?;
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    let undelegates = sqlx::query_as::<_, Undelegate>(
        r"
        SELECT id, tx_hash, validator_identity_key,
               delegation_amount::TEXT as delegation_amount,
               unbonded_amount::TEXT as unbonded_amount,
               epoch_index, unbonding_start_height, release_height,
               block_height, timestamp, claimed
        FROM undelegates
        WHERE claimed = FALSE
        ORDER BY release_height ASC
        LIMIT $1 OFFSET $2
        ",
    )
    .bind(i64::from(limit))
    .bind(i64::from(offset))
    .fetch_all(db)
    .await?;

    Ok(undelegates)
}

pub async fn resolve_undelegations_releasing_soon(
    ctx: &Context<'_>,
    current_height: i64,
    blocks_ahead: Option<i32>,
) -> async_graphql::Result<Vec<Undelegate>> {
    let db = ctx.data::<PgPool>()?;
    let blocks_ahead = i64::from(blocks_ahead.unwrap_or(7200));

    let undelegates = sqlx::query_as::<_, Undelegate>(
        r"
        SELECT id, tx_hash, validator_identity_key,
               delegation_amount::TEXT as delegation_amount,
               unbonded_amount::TEXT as unbonded_amount,
               epoch_index, unbonding_start_height, release_height,
               block_height, timestamp, claimed
        FROM undelegates
        WHERE claimed = FALSE
        AND release_height BETWEEN $1 AND $2
        ORDER BY release_height ASC
        ",
    )
    .bind(current_height)
    .bind(current_height + blocks_ahead)
    .fetch_all(db)
    .await?;

    Ok(undelegates)
}
