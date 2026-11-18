//! DebitMyData's reference price, block by block: what the electricity it takes to mine one DMD
//! costs, never below DebitMyData's minimum rate. DMD has no market price yet; this is the price
//! the DebitMyData Agent Gateway charges with, by its formula (`pricing/mining_cost.py`) and its
//! approved settings (`config/payment_routes.yaml`, `dmd_mining_cost`):
//!
//! ```text
//! USD per DMD = hashes per block × joules per hash × USD per joule ÷ DMD per block
//! price       = the higher of that and the minimum rate
//! ```
//!
//! Hashes per block: the chain's work over the last 4,032 blocks (a week) up to the block,
//! divided by their number, so a short burst of hashing power barely moves it. DMD per block: the
//! block's reward (35 DMD, halving every 300,000 blocks), fees left out.
//!
//! Each block is priced with the settings of its own time ([`SETTINGS`]): when DebitMyData changes
//! them, a new entry starts at that date and earlier blocks keep their prices.

use brk_types::{CENTS_PER_DOLLAR, Cents, Height, Timestamp};

/// Blocks the work is averaged over: one week of 2.5-minute blocks (the gateway's).
pub const WINDOW_BLOCKS: usize = 4_032;

/// The formula's settings from a date on.
pub struct Settings {
    /// The first block time these settings price, in seconds since 1970 (UTC).
    pub from: u32,
    /// What miners pay for electricity, in USD per kWh.
    pub usd_per_kwh: f64,
    /// Energy a SHA-256 machine uses per terahash, in joules.
    pub joules_per_terahash: f64,
    /// DebitMyData's minimum rate, in USD per DMD.
    pub minimum_usd_per_dmd: f64,
}

/// Oldest first. The gateway's values, reviewed every quarter: when they change there, add an
/// entry here from the date of the change (and bump the price plugin's storage version).
pub const SETTINGS: &[Settings] = &[Settings {
    from: 0,
    usd_per_kwh: 0.06,
    joules_per_terahash: 20.0,
    minimum_usd_per_dmd: 0.001,
}];

/// The settings that price a block made at `time`.
pub fn settings_at(time: Timestamp) -> &'static Settings {
    SETTINGS
        .iter()
        .rev()
        .find(|settings| settings.from <= *time)
        .unwrap_or(&SETTINGS[0])
}

/// The work a block of this difficulty proves: hashes to find it on average. Difficulty is
/// relative to the target 0xffff × 2^208, so the work is difficulty × 2^256 / (0xffff × 2^208),
/// as the node's `chainwork` adds it up.
pub fn work(difficulty: f64) -> f64 {
    difficulty * (2f64.powi(48) / 65_535.0)
}

/// The reference price of the block at `height`, from the total work of the blocks averaged
/// (`window_work` over `window_blocks` of them), its reward and its time. Before any block can
/// be averaged (the genesis block), the minimum rate.
pub fn price(height: Height, window_work: f64, window_blocks: usize, time: Timestamp) -> Cents {
    let settings = settings_at(time);
    let minimum = settings.minimum_usd_per_dmd;
    let dmd_per_block = u64::from(height.subsidy()) as f64 / 100_000_000.0;
    let cost = if window_blocks == 0 || dmd_per_block <= 0.0 {
        0.0
    } else {
        let hashes_per_block = window_work / window_blocks as f64;
        let kwh = hashes_per_block / 1e12 * settings.joules_per_terahash / 3_600_000.0;
        kwh * settings.usd_per_kwh / dmd_per_block
    };
    let usd = if cost.is_finite() {
        cost.max(minimum)
    } else {
        minimum
    };
    Cents::from((usd * CENTS_PER_DOLLAR as f64).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_minimum_until_the_mining_cost_is_higher() {
        // DMD's chain today: its mining cost is far below the minimum
        let at = Timestamp::new(1_790_000_000);
        let low = work(300_000.0);
        assert_eq!(
            price(Height::new(150_000), low * 4_032.0, 4_032, at),
            Cents::from(1_000u64)
        );
        // the genesis block: no work to average yet
        assert_eq!(price(Height::new(0), 0.0, 0, at), Cents::from(1_000u64));

        // 10^9 difficulty: 4.295 × 10^18 hashes a block, 23.86 kWh at 20 J/TH, $1.4317 at
        // $0.06/kWh, for 35 DMD: $0.040905 per DMD
        let high = work(1e9);
        assert_eq!(
            price(Height::new(150_000), high * 4_032.0, 4_032, at),
            Cents::from(40_905u64)
        );
        // after the first halving the same work buys half the DMD: twice the cost
        assert_eq!(
            price(Height::new(300_000), high * 4_032.0, 4_032, at),
            Cents::from(81_810u64)
        );
    }
}
