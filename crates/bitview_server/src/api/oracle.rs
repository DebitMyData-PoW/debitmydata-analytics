use aide::axum::{ApiRouter, routing::get_with};
use axum::{http::HeaderMap, response::Response};
use brk_types::Dollars;

use crate::{
    AppState, api::tags, extended::TransformResponseExtended, params::Empty,
    request_state::RequestState,
};

/// DMD's reference price at the tip block. Bitview's oracle routes (`/api/oracle/...`) are not
/// served: the oracle reads Bitcoin's round-dollar payments, which DebitMyData's chain does not
/// have.
async fn serve_price(headers: HeaderMap, _: Empty, RequestState(state): RequestState) -> Response {
    state
        .respond_json_content(&headers, |query| query.live_price())
        .await
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/price",
        get_with(serve_price, |op| {
            op.id("get_price")
                .tag(tags::ORACLE)
                .summary("DMD reference price")
                .description(
                    "DMD's reference price in US dollars at the latest block: the cost of the \
                    electricity it takes to mine one DMD, never below $0.001 (DebitMyData's \
                    minimum rate). Not a market price. Every block's is at \
                    `GET /api/series/price/height`.",
                )
                .json_response::<Dollars>()
                .not_modified()
                .not_found()
        }),
    )
}
