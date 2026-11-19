use bitview_traversable::Traversable;
use brk_types::Bitcoin;
use vecdb::{DeltaChange, VecValue};

use crate::{AmountType, LazyDeltaFromHeight, LazyPerBlock};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDeltaAmountFromHeight<S, C>
where
    S: VecValue,
    C: AmountType,
{
    /// Reported in DMD; one DMD equals 100,000,000 data.
    pub(crate) btc: LazyPerBlock<Bitcoin, C>,
    /// Reported in data.
    #[traversable(hidden)]
    pub(crate) sats: LazyDeltaFromHeight<S, C, DeltaChange>,
}
