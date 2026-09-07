use cometindex::PgTransaction;
use serde_json::Value;
use sqlx::types::chrono::{DateTime, Utc};

use crate::parsing::identity_key_to_validator_address;

const UNBONDING_BLOCKS: i64 = 120_960;

pub async fn process_delegate_actions(
    dbtx: &mut PgTransaction<'_>,
    tx_hash: [u8; 32],
    tx_json: &Value,
    block_height: u64,
    timestamp: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let Some(action_views) = tx_json
        .get("bodyView")
        .and_then(|bv| bv.get("actionViews"))
        .and_then(|av| av.as_array())
    else {
        return Ok(());
    };

    for action_view in action_views {
        if let Some(delegate) = action_view.get("delegate") {
            if let Err(e) = index_delegate(dbtx, tx_hash, delegate, block_height, timestamp).await {
                tracing::error!(
                    "failed to index delegate action in tx {}: {}",
                    hex::encode(tx_hash),
                    e
                );
            }
        }
    }

    Ok(())
}

pub async fn process_undelegate_actions(
    dbtx: &mut PgTransaction<'_>,
    tx_hash: [u8; 32],
    tx_json: &Value,
    block_height: u64,
    timestamp: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let Some(action_views) = tx_json
        .get("bodyView")
        .and_then(|bv| bv.get("actionViews"))
        .and_then(|av| av.as_array())
    else {
        return Ok(());
    };

    for action_view in action_views {
        if let Some(undelegate) = action_view.get("undelegate") {
            if let Err(e) =
                index_undelegate(dbtx, tx_hash, undelegate, block_height, timestamp).await
            {
                tracing::error!(
                    "failed to index undelegate action in tx {}: {}",
                    hex::encode(tx_hash),
                    e
                );
            }
        }
    }

    Ok(())
}

async fn index_delegate(
    dbtx: &mut PgTransaction<'_>,
    tx_hash: [u8; 32],
    delegate: &Value,
    block_height: u64,
    timestamp: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let validator_ik = delegate
        .get("validatorIdentity")
        .and_then(|v| v.get("ik"))
        .and_then(|ik| ik.as_str());

    let delegation_amount = delegate
        .get("delegationAmount")
        .and_then(|da| da.get("lo"))
        .and_then(|lo| lo.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let unbonded_amount = delegate
        .get("unbondedAmount")
        .and_then(|ua| ua.get("lo"))
        .and_then(|lo| lo.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let epoch_index = delegate
        .get("fromEpoch")
        .and_then(|fe| fe.get("index"))
        .and_then(|idx| idx.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let (validator_ik, delegation_amount, unbonded_amount, epoch_index) = match (
        validator_ik,
        delegation_amount,
        unbonded_amount,
        epoch_index,
    ) {
        (Some(ik), Some(da), Some(ua), Some(ei)) => (ik, da, ua, ei),
        _ => {
            tracing::debug!("incomplete delegate action data, skipping");
            return Ok(());
        }
    };

    let validator_address = match identity_key_to_validator_address(validator_ik) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("failed to decode validator identity key: {}", e);
            return Ok(());
        }
    };

    let block_height_i64 = i64::try_from(block_height).unwrap_or(0);

    sqlx::query(
        r"
        INSERT INTO delegates
        (tx_hash, validator_identity_key, delegation_amount, unbonded_amount,
         epoch_index, block_height, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (tx_hash, validator_identity_key) DO NOTHING
        ",
    )
    .bind(tx_hash.as_ref())
    .bind(&validator_address)
    .bind(delegation_amount)
    .bind(unbonded_amount)
    .bind(epoch_index)
    .bind(block_height_i64)
    .bind(timestamp)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn index_undelegate(
    dbtx: &mut PgTransaction<'_>,
    tx_hash: [u8; 32],
    undelegate: &Value,
    block_height: u64,
    timestamp: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let validator_ik = undelegate
        .get("validatorIdentity")
        .and_then(|v| v.get("ik"))
        .and_then(|ik| ik.as_str());

    let delegation_amount = undelegate
        .get("delegationAmount")
        .and_then(|da| da.get("lo"))
        .and_then(|lo| lo.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let unbonded_amount = undelegate
        .get("unbondedAmount")
        .and_then(|ua| ua.get("lo"))
        .and_then(|lo| lo.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let epoch_index = undelegate
        .get("fromEpoch")
        .and_then(|fe| fe.get("index"))
        .and_then(|idx| idx.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let unbonding_start_height = undelegate
        .get("fromEpoch")
        .and_then(|fe| fe.get("startHeight"))
        .and_then(|sh| sh.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let (validator_ik, delegation_amount, unbonded_amount, epoch_index, unbonding_start_height) =
        match (
            validator_ik,
            delegation_amount,
            unbonded_amount,
            epoch_index,
            unbonding_start_height,
        ) {
            (Some(ik), Some(da), Some(ua), Some(ei), Some(ush)) => (ik, da, ua, ei, ush),
            _ => {
                tracing::debug!("incomplete undelegate action data, skipping");
                return Ok(());
            }
        };

    let validator_address = match identity_key_to_validator_address(validator_ik) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("failed to decode validator identity key: {}", e);
            return Ok(());
        }
    };

    let block_height_i64 = i64::try_from(block_height).unwrap_or(0);
    let release_height = unbonding_start_height + UNBONDING_BLOCKS;

    sqlx::query(
        r"
        INSERT INTO undelegates
        (tx_hash, validator_identity_key, delegation_amount, unbonded_amount,
         epoch_index, unbonding_start_height, release_height, block_height, timestamp, claimed)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, FALSE)
        ON CONFLICT (tx_hash, validator_identity_key, unbonding_start_height) DO NOTHING
        ",
    )
    .bind(tx_hash.as_ref())
    .bind(&validator_address)
    .bind(delegation_amount)
    .bind(unbonded_amount)
    .bind(epoch_index)
    .bind(unbonding_start_height)
    .bind(release_height)
    .bind(block_height_i64)
    .bind(timestamp)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

pub async fn mark_undelegate_claimed(
    dbtx: &mut PgTransaction<'_>,
    tx_json: &Value,
) -> Result<(), sqlx::Error> {
    let Some(action_views) = tx_json
        .get("bodyView")
        .and_then(|bv| bv.get("actionViews"))
        .and_then(|av| av.as_array())
    else {
        return Ok(());
    };

    for action_view in action_views {
        if let Some(claim) = action_view.get("undelegateClaim") {
            if let Some(body) = claim.get("body") {
                let validator_ik = body
                    .get("validatorIdentity")
                    .and_then(|v| v.get("ik"))
                    .and_then(|ik| ik.as_str());

                let unbonding_start_height = body
                    .get("unbondingStartHeight")
                    .and_then(|h| h.as_str())
                    .and_then(|s| s.parse::<i64>().ok());

                if let (Some(ik), Some(start_height)) = (validator_ik, unbonding_start_height) {
                    if let Ok(validator_address) = identity_key_to_validator_address(ik) {
                        let _ = sqlx::query(
                            r"
                            UPDATE undelegates
                            SET claimed = TRUE
                            WHERE validator_identity_key = $1
                            AND unbonding_start_height = $2
                            AND claimed = FALSE
                            ",
                        )
                        .bind(&validator_address)
                        .bind(start_height)
                        .execute(dbtx.as_mut())
                        .await;
                    }
                }
            }
        }
    }

    Ok(())
}
