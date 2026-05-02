// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    delegates (id) {
        id -> Int8,
        tx_hash -> Bytea,
        validator_identity_key -> Text,
        delegation_amount -> Numeric,
        unbonded_amount -> Numeric,
        epoch_index -> Int8,
        block_height -> Int8,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    dex_batch_swaps (id) {
        id -> Int4,
        block_height -> Int8,
        block_timestamp -> Timestamptz,
        execution_type -> Text,
        total_input_amount -> Numeric,
        total_input_asset_id -> Text,
        total_output_amount -> Numeric,
        total_output_asset_id -> Text,
        individual_swaps_count -> Int4,
        raw_execution_data -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    dex_individual_swap_routes (id) {
        id -> Int4,
        individual_swap_id -> Int4,
        route_step -> Int4,
        amount -> Numeric,
        asset_id -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    dex_individual_swaps (id) {
        id -> Int4,
        batch_swap_id -> Int4,
        swap_index -> Int4,
        input_amount -> Numeric,
        input_asset_id -> Text,
        output_amount -> Numeric,
        output_asset_id -> Text,
        route_steps_count -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    dex_liquidity_positions (position_id) {
        position_id -> Text,
        decoded_position_id -> Text,
        trading_pair_asset1 -> Text,
        trading_pair_asset2 -> Text,
        reserves1_amount -> Numeric,
        reserves2_amount -> Numeric,
        state -> Text,
        fee_percentage -> Numeric,
        created_height -> Int8,
        created_at -> Timestamptz,
        updated_height -> Int8,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    explorer_assets (asset_id) {
        asset_id -> Text,
        decoded_passet -> Text,
        first_seen_height -> Int8,
        first_seen_time -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    governance_parameters (chain_id) {
        chain_id -> Text,
        deposit_amount -> Nullable<Numeric>,
        passing_threshold -> Nullable<Numeric>,
        slashing_threshold -> Nullable<Numeric>,
        valid_quorum -> Nullable<Numeric>,
        proposal_voting_blocks -> Nullable<Int8>,
        updated_height -> Int8,
        updated_at -> Timestamptz,
        raw_params -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    governance_proposals (proposal_id) {
        proposal_id -> Int8,
        title -> Text,
        description -> Nullable<Text>,
        kind -> Text,
        state -> Text,
        outcome -> Nullable<Text>,
        deposit_amount -> Nullable<Numeric>,
        start_block_height -> Nullable<Int8>,
        end_block_height -> Nullable<Int8>,
        start_timestamp -> Nullable<Timestamptz>,
        end_timestamp -> Nullable<Timestamptz>,
        quorum -> Nullable<Numeric>,
        payload -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    governance_votes (id) {
        id -> Int8,
        proposal_id -> Int8,
        validator_identity_key -> Nullable<Text>,
        vote -> Text,
        voting_power -> Nullable<Numeric>,
        effective_voting_power -> Nullable<Numeric>,
        voting_power_percentage -> Nullable<Numeric>,
        parent_validator_identity_key -> Nullable<Text>,
        block_height -> Int8,
        voted_at -> Timestamptz,
        tx_hash -> Nullable<Bytea>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    ibc_transfers (id) {
        id -> Int8,
        client_id -> Text,
        channel_id -> Text,
        direction -> Text,
        amount -> Int8,
        asset_id -> Nullable<Bytea>,
        timestamp -> Timestamptz,
        tx_hash -> Nullable<Bytea>,
        status -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    ibc_withdrawals (id) {
        id -> Int8,
        tx_hash -> Bytea,
        asset_id -> Text,
        amount -> Numeric,
        denom -> Text,
        destination_chain_address -> Text,
        source_channel -> Nullable<Text>,
        return_address -> Nullable<Text>,
        timeout_height -> Nullable<Int8>,
        timeout_timestamp -> Nullable<Int8>,
        use_compat_address -> Nullable<Bool>,
        block_height -> Int8,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    undelegates (id) {
        id -> Int8,
        tx_hash -> Bytea,
        validator_identity_key -> Text,
        delegation_amount -> Numeric,
        unbonded_amount -> Numeric,
        epoch_index -> Int8,
        unbonding_start_height -> Int8,
        release_height -> Int8,
        block_height -> Int8,
        timestamp -> Timestamptz,
        claimed -> Nullable<Bool>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    validator_voting_power_history (id) {
        id -> Int8,
        validator_identity_key -> Text,
        voting_power -> Int8,
        block_height -> Int8,
        timestamp -> Timestamptz,
        epoch_index -> Nullable<Int8>,
    }
}

diesel::joinable!(dex_individual_swap_routes -> dex_individual_swaps (individual_swap_id));
diesel::joinable!(dex_individual_swap_routes -> explorer_assets (asset_id));
diesel::joinable!(dex_individual_swaps -> dex_batch_swaps (batch_swap_id));
diesel::joinable!(governance_votes -> governance_proposals (proposal_id));

diesel::allow_tables_to_appear_in_same_query!(
    delegates,
    dex_batch_swaps,
    dex_individual_swap_routes,
    dex_individual_swaps,
    dex_liquidity_positions,
    explorer_assets,
    governance_parameters,
    governance_proposals,
    governance_votes,
    ibc_transfers,
    ibc_withdrawals,
    undelegates,
    validator_voting_power_history,
);
