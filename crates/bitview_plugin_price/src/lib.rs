mod compute;
mod dependencies;
mod endpoint;
mod has;
mod import;
mod ohlc;
mod oracle_feed;
pub mod reference;
mod split;

pub use dependencies::Dependencies;
pub use has::HasPrice;
pub use oracle_feed::{block_payments, block_tally, feed_blocks_for_warmup, feed_blocks_with};

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Float64;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock, Price};
use brk_oracle::{Oracle, VERSION as ORACLE_VERSION};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, PcoVec, Rw, StorageMode};

use crate::{ohlc::OhlcPrice, split::SplitPrice};

// 100: DebitMyData's reference price instead of the oracle (`reference`).
const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("price"), Version::new(100 + ORACLE_VERSION));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Separate open, high, low, and close views of the block-level DMD/USD reference
    /// price for each supported time period. The reference price is DebitMyData's: the cost
    /// of the electricity it takes to mine one DMD, never below $0.001. Not a market price.
    #[traversable(flatten)]
    split: SplitPrice,
    /// Open-high-low-close (OHLC) candles formed from block-level DMD/USD reference
    /// prices within each supported time period. Empty periods carry the previous close as
    /// all four candle values.
    #[traversable(lead = "Candles: each period's opening, highest, lowest and closing prices.")]
    ohlc: OhlcPrice,
    /// DebitMyData's DMD/USD reference price at each block: the cost of the electricity
    /// it takes to mine one DMD (the chain's work over the last 4,032 blocks, at 20 J/TH and
    /// $0.06/kWh, divided by the block reward), never below DebitMyData's minimum rate of
    /// $0.001. The DebitMyData Agent Gateway's price, worked out for every block from the
    /// chain alone. DMD has no market price yet: this is not one, and amounts in US dollars
    /// valued with it are not market values. At time-period indexes, the value is taken at
    /// the period's final block; null when the period has no block.
    #[traversable(lead = "DMD's reference price in US dollars, block by block: the cost of mining one DMD, at least $0.001. Not a market price.")]
    pub spot: Price<PerBlock<Cents, M>>,
    /// Whole sats (10^-8 DMD) one US dollar buys at the reference price, rounded down:
    /// 100,000,000 divided by the price in USD per DMD. 0 while the price is zero.
    #[traversable(lead = "Sats per dollar: how many 10^-8 DMD units one US dollar buys at the reference price.")]
    sats_per_dollar: LazyPerBlock<Sats, Cents>,
    /// The price on the oracle's log scale at each height. Kept for Bitview's format; the
    /// oracle does not price DebitMyData's blocks.
    #[traversable(hidden)]
    pub oracle_bin: M::Stored<PcoVec<Height, Float64>>,
    /// Unused on DebitMyData (always `None`): the reference price needs no oracle.
    oracle: M::WriteOnly<Option<(usize, Oracle)>>,
}
