use brk_types::{Cents, Sats};
use vecdb::UnaryTransform;

/// A price (millionths of a dollar per DMD) to the whole sats one dollar buys: none at a zero price.
pub struct CentsToSatsPerDollar;

impl UnaryTransform<Cents, Option<Sats>> for CentsToSatsPerDollar {
    #[inline(always)]
    fn apply(cents: Cents) -> Option<Sats> {
        cents.sats_per_dollar()
    }
}
