use axum::{extract::Query, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub(super) struct DateRangeQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<i64>,
}

pub(super) async fn get_cost_summary(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    match state.store.cost_store.query_summary(None).await {
        Ok(summary) => Json(json!(summary)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(super) async fn get_daily_tokens(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> impl IntoResponse {
    let start = q.start.as_deref().unwrap_or("1970-01-01");
    let end = q.end.as_deref().unwrap_or("9999-12-31");

    match state.store.cost_store.query_daily_tokens(start, end).await {
        Ok(rows) => Json(json!({ "data": rows })).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(super) async fn get_tool_stats(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> impl IntoResponse {
    let start = q.start.as_deref().unwrap_or("1970-01-01");
    let end = q.end.as_deref().unwrap_or("9999-12-31");

    match state.store.cost_store.query_tool_stats(start, end).await {
        Ok(rows) => Json(json!({ "data": rows })).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(super) async fn get_session_costs(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);

    match state.store.cost_store.query_sessions(limit).await {
        Ok(rows) => Json(json!({ "data": rows })).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
