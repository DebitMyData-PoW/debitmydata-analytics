use bitview_traversable::Traversable;

/// A DMD amount and its fiat value, with independently chosen representations.
/// Series expose DMD and USD over their sats and cents.
#[derive(Clone, Traversable)]
pub struct Value<S, C, B, U> {
    /// Reported in DMD; one DMD equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars, each block's amount valued at that block's spot price.
    pub usd: U,
    /// Reported in millionths of a US dollar (Bitview's cents); 1,000,000 make one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}

/// A DMD amount at the spot price, in DMD, USD and satoshis.
#[derive(Clone, Traversable)]
pub struct SpotValue<S, B, U> {
    /// Reported in DMD; one DMD equals 100,000,000 satoshis.
    pub btc: B,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub sats: S,
    /// Reported in US dollars: the amount at the spot price (its market cap).
    #[traversable(lead = "Its market value: valued at the price of the day.")]
    pub usd: U,
}
