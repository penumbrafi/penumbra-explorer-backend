use crate::api::graphql::context::ApiContext;
use crate::api::graphql::types::ibc::{IbcFlowHistory, Stats, TotalShieldedVolume};
use crate::api::graphql::types::inputs::TimePeriod;
use async_graphql::{Context, Result};
use sqlx::Row;

/// Resolves IBC stats with optional filtering
///
/// # Errors
/// Returns an error if the database query fails
pub async fn resolve_ibc_stats(
    ctx: &Context<'_>,
    client_id: Option<String>,
    time_period: Option<TimePeriod>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Stats>> {
    let period_str = match time_period {
        Some(TimePeriod::DAY) => Some("24h".to_string()),
        Some(TimePeriod::MONTH) => Some("30d".to_string()),
        Some(TimePeriod::ALL) | None => None,
    };

    Stats::get_all(ctx, client_id, period_str, limit, offset).await
}

/// Resolves total shielded volume across all IBC clients
///
/// # Errors
/// Returns an error if the database query fails
pub async fn resolve_total_shielded_volume(ctx: &Context<'_>) -> Result<TotalShieldedVolume> {
    TotalShieldedVolume::get(ctx).await
}

/// Resolves IBC inflow/outflow history as daily time series
///
/// # Errors
/// Returns an error if the database query fails
pub async fn resolve_ibc_flow_history(
    ctx: &Context<'_>,
    client_id: Option<String>,
    days: Option<i32>,
) -> Result<Vec<IbcFlowHistory>> {
    let db = &ctx.data_unchecked::<ApiContext>().db;
    let days = days.unwrap_or(30).min(365);

    let mut where_clauses =
        vec!["t.timestamp > NOW() - make_interval(days => $1::int)".to_string()];
    let mut param_idx = 1;

    if client_id.is_some() {
        param_idx += 1;
        where_clauses.push(format!("t.client_id = ${param_idx}"));
    }

    let where_sql = where_clauses.join(" AND ");

    let query = format!(
        r"
        SELECT
            DATE_TRUNC('day', t.timestamp)::DATE::TEXT AS date,
            COALESCE(SUM(CASE WHEN t.direction = 'inbound' AND t.status = 'completed'
                THEN t.amount ELSE 0 END), 0)::TEXT AS inflow_volume,
            COALESCE(SUM(CASE WHEN t.direction = 'outbound' AND t.status = 'completed'
                THEN t.amount ELSE 0 END), 0)::TEXT AS outflow_volume,
            COUNT(*) FILTER (WHERE t.direction = 'inbound' AND t.status = 'completed')
                AS inflow_count,
            COUNT(*) FILTER (WHERE t.direction = 'outbound' AND t.status = 'completed')
                AS outflow_count
        FROM ibc_transfers t
        WHERE {where_sql}
        GROUP BY DATE_TRUNC('day', t.timestamp)
        ORDER BY date ASC
        "
    );

    let mut qb = sqlx::query(&query).bind(days);
    if let Some(cid) = client_id {
        qb = qb.bind(cid);
    }

    let rows = qb.fetch_all(db).await?;

    Ok(rows
        .into_iter()
        .map(|row| IbcFlowHistory {
            date: row.get("date"),
            inflow_volume: row.get("inflow_volume"),
            outflow_volume: row.get("outflow_volume"),
            inflow_count: row.get("inflow_count"),
            outflow_count: row.get("outflow_count"),
        })
        .collect())
}
