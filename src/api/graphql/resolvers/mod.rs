mod block;
mod dex;
mod governance;
mod ibc;
mod search;
mod staking;
mod stats;
mod subscription;
mod transaction;
mod validator;

use crate::api::graphql::types::inputs::TimePeriod;
use async_graphql::Object;

pub use block::{get as resolve_block, resolve_blocks_collection};
pub use dex::{
    resolve_dex_stats, resolve_latest_executions, resolve_liquidity_positions,
    resolve_recent_swap_prices, resolve_swap_volume_history, resolve_trading_pair_liquidity,
    resolve_trading_volume_24h,
};
pub use governance::{
    resolve_active_proposals, resolve_governance_parameters, resolve_past_proposals,
    resolve_proposal_detail, resolve_vote_for_transaction,
};
pub use ibc::{resolve_ibc_flow_history, resolve_ibc_stats, resolve_total_shielded_volume};
pub use search::resolve_search;
pub use staking::{
    resolve_pending_undelegations, resolve_undelegations_releasing_soon,
    resolve_validator_delegates, resolve_validator_staking_stats, resolve_validator_undelegates,
};
pub use stats::resolve_stats;
pub use subscription::Root as SubscriptionRoot;
pub use transaction::{resolve_transaction, resolve_transactions_collection};
pub use validator::{resolve_validator_details, resolve_validator_voting_power_history, resolve_validators_homepage};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn block(
        &self,
        ctx: &async_graphql::Context<'_>,
        height: i32,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::Block>> {
        resolve_block(ctx, height).await
    }

    async fn blocks(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: crate::api::graphql::types::CollectionLimit,
        filter: Option<crate::api::graphql::types::BlockFilter>,
    ) -> async_graphql::Result<crate::api::graphql::types::BlockCollection> {
        resolve_blocks_collection(ctx, limit, filter).await
    }

    async fn transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
        hash: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::Transaction>> {
        resolve_transaction(ctx, hash).await
    }

    async fn transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: crate::api::graphql::types::CollectionLimit,
        filter: Option<crate::api::graphql::types::TransactionFilter>,
    ) -> async_graphql::Result<crate::api::graphql::types::TransactionCollection> {
        resolve_transactions_collection(ctx, limit, filter).await
    }

    async fn search(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::SearchResult>> {
        resolve_search(ctx, slug).await
    }

    async fn stats(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<crate::api::graphql::types::Stats> {
        resolve_stats(ctx).await
    }

    async fn db_block(
        &self,
        ctx: &async_graphql::Context<'_>,
        height: i64,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::DbBlock>> {
        crate::api::graphql::types::DbBlock::get_by_height(ctx, height).await
    }

    async fn db_blocks(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::DbBlock>> {
        crate::api::graphql::types::DbBlock::get_all(ctx, limit, offset).await
    }

    async fn db_latest_block(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::DbBlock>> {
        crate::api::graphql::types::DbBlock::get_latest(ctx).await
    }

    async fn db_raw_transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
        tx_hash_hex: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::DbRawTransaction>> {
        crate::api::graphql::types::DbRawTransaction::get_by_hash(ctx, tx_hash_hex).await
    }

    async fn db_raw_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::DbRawTransaction>> {
        crate::api::graphql::types::DbRawTransaction::get_all(ctx, limit, offset, None).await
    }

    async fn ibc_stats(
        &self,
        ctx: &async_graphql::Context<'_>,
        client_id: Option<String>,
        time_period: Option<TimePeriod>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::IbcStats>> {
        resolve_ibc_stats(ctx, client_id, time_period, limit, offset).await
    }

    async fn ibc_flow_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        client_id: Option<String>,
        days: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::ibc::IbcFlowHistory>> {
        resolve_ibc_flow_history(ctx, client_id, days).await
    }

    async fn ibc_total_shielded_volume(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<crate::api::graphql::types::ibc::TotalShieldedVolume> {
        resolve_total_shielded_volume(ctx).await
    }

    async fn validators_homepage(
        &self,
        ctx: &async_graphql::Context<'_>,
        filter: Option<crate::api::graphql::types::ValidatorFilter>,
    ) -> async_graphql::Result<crate::api::graphql::types::ValidatorHomepageData> {
        resolve_validators_homepage(ctx, filter).await
    }

    async fn validator_details(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::validator::ValidatorDetails>>
    {
        resolve_validator_details(ctx, id).await
    }

    async fn validator_voting_power_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        validator_id: String,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::validator::VotingPowerHistoryEntry>> {
        resolve_validator_voting_power_history(ctx, validator_id, start_time, end_time, limit).await
    }

    async fn liquidity_positions(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: crate::api::graphql::types::CollectionLimit,
        filter: Option<crate::api::graphql::types::LiquidityPositionFilter>,
    ) -> async_graphql::Result<crate::api::graphql::types::LiquidityPositionCollection> {
        resolve_liquidity_positions(ctx, limit, filter).await
    }

    async fn latest_executions(
        &self,
        ctx: &async_graphql::Context<'_>,
        filter: Option<crate::api::graphql::types::SwapExecutionFilter>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::SwapExecution>> {
        resolve_latest_executions(ctx, filter).await
    }

    async fn dex_stats(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<crate::api::graphql::types::DexStats> {
        resolve_dex_stats(ctx).await
    }

    async fn trading_pair_liquidity(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::TradingPairLiquidity>> {
        resolve_trading_pair_liquidity(ctx, limit).await
    }

    async fn trading_volume_24h(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::TradingVolume24h>> {
        resolve_trading_volume_24h(ctx, limit).await
    }

    async fn recent_swap_prices(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::RecentSwapPrice>> {
        resolve_recent_swap_prices(ctx, limit).await
    }

    async fn swap_volume_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        days: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::SwapVolumeHistory>> {
        resolve_swap_volume_history(ctx, days).await
    }

    async fn governance_parameters(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::GovernanceParameters>> {
        resolve_governance_parameters(ctx).await
    }

    async fn past_proposals(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: crate::api::graphql::types::CollectionLimit,
    ) -> async_graphql::Result<crate::api::graphql::types::PastProposalCollection> {
        resolve_past_proposals(ctx, limit).await
    }

    async fn active_proposals(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::ActiveProposal>> {
        resolve_active_proposals(ctx).await
    }

    async fn proposal_detail(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: i64,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::ProposalDetail>> {
        resolve_proposal_detail(ctx, id).await
    }

    #[graphql(name = "getVoteForTransaction")]
    async fn get_vote_for_transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
        tx_hash: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::VoteForTransaction>> {
        resolve_vote_for_transaction(ctx, tx_hash).await
    }

    async fn validator_delegates(
        &self,
        ctx: &async_graphql::Context<'_>,
        validator_id: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::Delegate>> {
        resolve_validator_delegates(ctx, validator_id, limit, offset).await
    }

    async fn validator_undelegates(
        &self,
        ctx: &async_graphql::Context<'_>,
        validator_id: String,
        limit: Option<i32>,
        offset: Option<i32>,
        pending_only: Option<bool>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::Undelegate>> {
        resolve_validator_undelegates(ctx, validator_id, limit, offset, pending_only).await
    }

    async fn validator_staking_stats(
        &self,
        ctx: &async_graphql::Context<'_>,
        validator_id: String,
    ) -> async_graphql::Result<Option<crate::api::graphql::types::ValidatorStakingStats>> {
        resolve_validator_staking_stats(ctx, validator_id).await
    }

    async fn pending_undelegations(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::Undelegate>> {
        resolve_pending_undelegations(ctx, limit, offset).await
    }

    async fn undelegations_releasing_soon(
        &self,
        ctx: &async_graphql::Context<'_>,
        current_height: i64,
        blocks_ahead: Option<i32>,
    ) -> async_graphql::Result<Vec<crate::api::graphql::types::Undelegate>> {
        resolve_undelegations_releasing_soon(ctx, current_height, blocks_ahead).await
    }
}
