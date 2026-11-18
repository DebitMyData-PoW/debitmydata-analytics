use bitview_compute::{prepare_computed, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::Float64;
use brk_error::Result;
use brk_exit::Exit;
use brk_oracle::cents_to_bin;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, WritableVec};

use crate::{
    Dependencies, Vecs,
    reference::{self, WINDOW_BLOCKS},
};

/// Bump when the reference price's formula or settings change.
const REFERENCE_VERSION: Version = Version::new(1);

/// A block's work as a whole number, so the window's total stays exact as blocks enter and leave.
fn work(difficulty: f64) -> u128 {
    reference::work(difficulty).round() as u128
}

impl Vecs {
    /// DebitMyData's reference price for every block not priced yet (see [`reference`]).
    fn compute_prices(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.compute_start().height;
        let blocks = &indexer.vecs().blocks;
        let source_version =
            blocks.difficulty.version() + blocks.timestamp.version() + REFERENCE_VERSION;
        let total = blocks.timestamp.len();
        prepare_computed(
            [
                &mut self.spot.cents.height as &mut dyn AnyStoredVec,
                &mut self.oracle_bin,
            ],
            source_version,
            starting_height.to_usize(),
            exit,
        )?;
        self.oracle = None;

        let start = self.spot.cents.height.len();
        if start >= total {
            return Ok(());
        }
        // The window of block `h`: the blocks after `h - WINDOW_BLOCKS` up to `h`, the genesis
        // block's work left out, as the node's chainwork difference gives it.
        let first = start.saturating_sub(WINDOW_BLOCKS).max(1);
        let difficulties = blocks.difficulty.collect_range_at(first, total);
        let timestamps = blocks.timestamp.collect_range_at(start, total);
        let work_at = |height: usize| work(*difficulties[height - first]);
        let mut window: u128 = (first..start).map(work_at).sum();

        for height in start..total {
            if height >= 1 {
                window += work_at(height);
            }
            if height > WINDOW_BLOCKS {
                window -= work_at(height - WINDOW_BLOCKS);
            }
            let cents = reference::price(
                Height::from(height),
                window as f64,
                height.min(WINDOW_BLOCKS),
                timestamps[height - start],
            );
            self.spot.cents.height.push(cents);
            self.oracle_bin
                .push(Float64::from(cents_to_bin(f64::from(cents))));
        }
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        self.compute_prices(indexer, exit)?;
        write_locked(
            [
                &mut self.spot.cents.height as &mut dyn AnyStoredVec,
                &mut self.oracle_bin,
            ],
            exit,
        )
    }
}
