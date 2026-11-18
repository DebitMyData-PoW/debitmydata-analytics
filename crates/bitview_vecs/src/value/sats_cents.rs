use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct SatsCents<S, C> {
    /// Reported in satoshis.
    pub sats: S,
    /// Reported in millionths of a US dollar (Bitview's cents); 1,000,000 make one US dollar.
    pub cents: C,
}
