use brk_types::{Cents, Dollars};

use crate::config::START_HEIGHT_SLOW;

/// First height with a price. On DebitMyData every block has one, from the genesis block: its
/// reference price (`bitview_plugin_price::reference`), not Bitcoin's exchange history.
pub const FIRST_PRICE_HEIGHT: usize = 0;

/// Pre-oracle dollar prices at each block's time, one per line, heights 0..START_HEIGHT_SLOW; from
/// height 337,525 (2015-01-05, Bitstamp's halt) Bitfinex minute closes, except the oracle's seed
/// (339,999), which keeps the price its measured path starts from.
const PRICES: &str = include_str!("prices.txt");

/// Baked pre-oracle prices starting at `start_height`, as a one-pass iterator.
pub fn pre_oracle_prices_from(start_height: usize) -> impl Iterator<Item = Cents> {
    PRICES
        .lines()
        .take(START_HEIGHT_SLOW)
        .skip(start_height)
        .map(parse_price_cents)
}

fn parse_price_cents(line: &str) -> Cents {
    Cents::from(Dollars::from(
        line.parse::<f64>().expect("invalid baked oracle price"),
    ))
}
