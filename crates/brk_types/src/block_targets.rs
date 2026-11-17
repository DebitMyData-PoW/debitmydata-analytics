pub const TARGET_BLOCKS_PER_DAY_F64: f64 = TARGET_BLOCKS_PER_DAY as f64;
pub const TARGET_BLOCKS_PER_DAY_F32: f32 = TARGET_BLOCKS_PER_DAY as f32;
/// A block every 2.5 minutes on DebitMyData: 576 a day.
pub const TARGET_BLOCKS_PER_DAY: u64 = 86_400 / crate::chain::TARGET_BLOCK_SECONDS;
