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
        HalvingEpochId::ALL
            .get(usize::from(HalvingEpoch::from(height)).min(last))
            .map(|id| id.select_mut(self))
    }
}

impl HalvingEpochId {
    pub const fn cohort(self) -> CohortId {
        CohortId::HalvingEpoch(self)
    }
}

/// What the creation-period cohorts (vintages, halving epochs) need for a block at `height` and
/// `timestamp`, if anything. They're listed one period ahead: a block in the last listed year or
/// epoch means the next must be added to the tree before it begins; past it, coins of the period
/// go to no cohort of the kind.
pub fn creation_cohorts_ahead(height: Height, timestamp: brk_types::Timestamp) -> Option<String> {
    let year = bitview_primitives::Date::from(timestamp).year();
    let epoch = usize::from(HalvingEpoch::from(height));
    let last_epoch = HalvingEpochId::ALL.len() - 1;
    // The first period the list lacks: the next one, or one already reached.
    let next_year = crate::VintageId::LAST_YEAR + 1;
    let next_epoch = last_epoch + 1;
    let mut missing = Vec::new();
    if year + 1 >= next_year {
        let when = if year >= next_year {
            "now: its coins are in none"
        } else {
            "before it begins"
        };
        missing.push(format!("vintage_{next_year} ({when})"));
    }
    if epoch + 1 >= next_epoch {
        let when = if epoch >= next_epoch {
            "now: its coins are in none"
        } else {
            "before it begins"
        };
        missing.push(format!(
            "halving_epoch_{next_epoch} (from height {}, {when})",
            next_epoch * brk_types::BLOCKS_PER_HALVING as usize
        ));
    }
    (!missing.is_empty()).then(|| format!("Creation-period cohorts to add: {}", missing.join("; ")))
}
