use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, SimpleObject, FromRow)]
pub struct Delegate {
    pub id: i64,
    #[sqlx(default)]
    pub tx_hash: Vec<u8>,
    pub validator_identity_key: String,
    #[graphql(name = "delegationAmount")]
    pub delegation_amount: String,
    #[graphql(name = "unbondedAmount")]
    pub unbonded_amount: String,
    #[graphql(name = "epochIndex")]
    pub epoch_index: i64,
    #[graphql(name = "blockHeight")]
    pub block_height: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, SimpleObject, FromRow)]
pub struct Undelegate {
    pub id: i64,
    #[sqlx(default)]
    pub tx_hash: Vec<u8>,
    pub validator_identity_key: String,
    #[graphql(name = "delegationAmount")]
    pub delegation_amount: String,
    #[graphql(name = "unbondedAmount")]
    pub unbonded_amount: String,
    #[graphql(name = "epochIndex")]
    pub epoch_index: i64,
    #[graphql(name = "unbondingStartHeight")]
    pub unbonding_start_height: i64,
    #[graphql(name = "releaseHeight")]
    pub release_height: i64,
    #[graphql(name = "blockHeight")]
    pub block_height: i64,
    pub timestamp: DateTime<Utc>,
    pub claimed: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ValidatorStakingStats {
    pub validator_identity_key: String,
    pub total_delegations: String,
    pub total_undelegations: String,
    pub pending_undelegations: String,
    pub pending_undelegate_count: i64,
    pub next_release_height: Option<i64>,
}
