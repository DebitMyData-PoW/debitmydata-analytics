use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Query, http::HeaderMap};
use bitview_types::{DifficultyAdjustment, HistoricalPrice, PricePoint};
use brk_types::Timestamp;

use super::historical_price;
use crate::{
    AppState,
    api::tags,
    extended::TransformResponseExtended,
    params::{Empty, HistoricalPriceParam},
    request_state::RequestState,
};

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/difficulty-adjustment",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_content(&headers, |q| {
                        q.difficulty_adjustment()
                    })
                    .await
            },
            |op| {
                op.id("get_difficulty_adjustment")
                    .tag(tags::GENERAL)
                    .summary("Difficulty adjustment")
                    .description("Progress through the current 2,016-block period and its estimates, as mempool.space reports a difficulty epoch. DebitMyData's difficulty itself adjusts at every block (LWMA): the period is Bitview's, for comparing.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*")
                    .json_response::<DifficultyAdjustment>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/prices",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state
                    .respond_json_content(&headers, |q| {
                        Ok(PricePoint {
                            time: Timestamp::now(),
                            usd: q.live_price()?,
                        })
                    })
                    .await
            },
            |op| {
                op.id("get_prices")
                    .tag(tags::GENERAL)
                    .summary("Current DMD price")
                    .description("DMD's reference price at the latest block (the cost of mining one DMD, at least $0.001; not a market price), USD only.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*")
                    .json_response::<PricePoint>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/historical-price",
        get_with(
            async |headers: HeaderMap,
                   Query(params): Query<HistoricalPriceParam>,
                   RequestState(state): RequestState| {
                historical_price::serve(state, headers, params.timestamp).await
            },
            |op| {
                op.id("get_historical_price")
                    .tag(tags::GENERAL)
                    .summary("Historical price")
                    .description("Completed four-hour closes of DMD's reference price (the cost of mining one DMD, at least $0.001; not a market price), newest first as mempool.space lists them, labeled by interval end, from the chain's first block. With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*")
                    .json_response::<HistoricalPrice>()
                    .not_modified()
            },
        ),
    )
}
