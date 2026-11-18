use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};

/// A fiat value represented in cents and US dollars.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct Fiat<C, U> {
    /// Reported in US dollars.
    pub usd: U,
    /// Reported in millionths of a US dollar (Bitview's cents); 1,000,000 make one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}

/// A dollar sum over a trailing window, saying when it has a value.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedSumUsd<U>(
    /// Defined for every window: every DebitMyData block has a price (its reference price).
    pub U,
);

/// A dollar level's change over a trailing window, saying when it has a value.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedDeltaUsd<U>(
    /// Defined for every window: every DebitMyData block has a price (its reference price).
    pub U,
);

/// A dollar mean over a trailing window, saying which blocks it's over.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedAvgUsd<U>(
    /// Over the window's blocks, each priced (every DebitMyData block has its reference price).
    pub U,
);
