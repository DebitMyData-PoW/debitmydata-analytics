use bitview_primitives::{Difficulty, Hashrate};
use vecdb::UnaryTransform;

pub struct DifficultyToHashrate;

impl UnaryTransform<Difficulty, Hashrate> for DifficultyToHashrate {
    #[inline(always)]
    fn apply(difficulty: Difficulty) -> Hashrate {
        // 2^32 hashes per unit of difficulty, a block every 150 seconds on DebitMyData
        const MULTIPLIER: f64 = 4_294_967_296.0 / brk_types::chain::TARGET_BLOCK_SECONDS as f64;
        Hashrate::from(*difficulty * MULTIPLIER)
    }
}
