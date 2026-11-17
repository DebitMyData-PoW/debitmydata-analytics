use bitview_primitives::HalvingEpoch;
#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

pub const HALVING_EPOCH_NAMES: ByHalvingEpoch<CohortName> = ByHalvingEpoch {
    _0: CohortName::new("halving_epoch_0", "0", "Halving epoch 0"),
    _1: CohortName::new("halving_epoch_1", "1", "Halving epoch 1"),
    _2: CohortName::new("halving_epoch_2", "2", "Halving epoch 2"),
    _3: CohortName::new("halving_epoch_3", "3", "Halving epoch 3"),
    _4: CohortName::new("halving_epoch_4", "4", "Halving epoch 4"),
    _5: CohortName::new("halving_epoch_5", "5", "Halving epoch 5"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByHalvingEpoch<T> {
    /// Uses UTXOs created during subsidy-halving epoch 0.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved before the first halving (blocks 0 to 299,999)."))]
    _0: T,
    /// Uses UTXOs created during subsidy-halving epoch 1.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved between the first and second halvings (blocks 300,000 to 599,999)."))]
    _1: T,
    /// Uses UTXOs created during subsidy-halving epoch 2.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved between the second and third halvings (blocks 600,000 to 899,999)."))]
    _2: T,
    /// Uses UTXOs created during subsidy-halving epoch 3.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved between the third and fourth halvings (blocks 900,000 to 1,199,999)."))]
    _3: T,
    /// Uses UTXOs created during subsidy-halving epoch 4.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved between the fourth and fifth halvings (blocks 1,200,000 to 1,499,999)."))]
    _4: T,
    /// Uses UTXOs created during subsidy-halving epoch 5 or any later one.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins last moved after the fifth halving (from block 1,500,000)."))]
    _5: T,
}

define_cohort_id!(
    HalvingEpochId for ByHalvingEpoch {
        _0 => _0,
        _1 => _1,
        _2 => _2,
        _3 => _3,
        _4 => _4,
        _5 => _5,
    }
);

impl<T> ByHalvingEpoch<T> {
    /// The cohort of coins created at `height`: its halving epoch, the last cohort for epoch 5
    /// and every later one (a test chain halves every 150 blocks and goes far past it).
    pub fn get_mut(&mut self, height: Height) -> Option<&mut T> {
        let last = HalvingEpochId::ALL.len() - 1;
        let halving = usize::from(HalvingEpoch::from(height)).min(last);
        HalvingEpochId::ALL
            .get(halving)
            .copied()
            .map(|id| id.select_mut(self))
    }
}

impl HalvingEpochId {
    pub const fn cohort(self) -> CohortId {
        CohortId::HalvingEpoch(self)
    }
}
