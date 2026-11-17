use bitview_primitives::{Count, Days, HalvingEpoch};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based halving epoch (block-subsidy era): block height divided by 300,000, rounded
    /// down; epoch zero begins at genesis. The reward is 35 DMD halved once per epoch, and starts
    /// again at 35 DMD after 32 epochs (9,600,000 blocks).
    #[traversable(lead = "Halving epoch: which subsidy era each block is in, one every 300,000 blocks.")]
    pub epoch: LazyPerBlock<HalvingEpoch>,
    /// Number of blocks from the represented height to the first block of the
    /// next subsidy era: 300,000 minus height modulo 300,000.
    #[traversable(lead = "Blocks until the next halving.")]
    pub blocks_to_halving: LazyPerBlock<Count>,
    /// Nominal days to the next subsidy halving, calculated as
    /// `blocks_to_halving / 576`; this does not use observed mining pace.
    #[traversable(lead = "Days until the next halving, at 2.5 minutes a block.")]
    pub days_to_halving: LazyPerBlock<Days, Count>,
}
