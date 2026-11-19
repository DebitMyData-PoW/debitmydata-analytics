use bitview_primitives::SupplyState;
use brk_types::{Height, Sats, TxIndex, Version};

// One schema version for every data source: bump it when stored data changes or a source is
// added.
pub const VERSION: Version = Version::new(49);

/// The newest blocks the first resume after a start reads back at their stored positions, about a
/// day of them: back from a crash, Core writes what it fetched since its last block-index write
/// (about hourly) again, in the order it comes back.
pub const RECHECKED_BLOCKS: usize = 144;

/// A BIP30 duplicate coinbase (https://github.com/bitcoin/bips/blob/master/bip-0030.mediawiki): it re-created
/// an earlier coinbase's txid and overwrote that coinbase's unspent outputs, as in Bitcoin Core.
/// The txid store keeps mapping the shared txid to the original.
#[allow(dead_code)]
struct DuplicateCoinbase {
    height: Height,
    tx_index: TxIndex,
    overwritten_height: Height,
    overwritten_tx_index: TxIndex,
}

/// None on DebitMyData: BIP34 (the height in each coinbase) applies from its first block, so no
/// coinbase can repeat an earlier one. Bitcoin's two, in 2010, are not DebitMyData's.
const DUPLICATE_COINBASES: [DuplicateCoinbase; 0] = [];

/// The output a BIP30 duplicate coinbase at `height` overwrote: the earlier coinbase's height and
/// its one lost output.
pub fn overwritten_output(height: Height) -> Option<(Height, SupplyState)> {
    let dup = DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.height == height)?;
    Some((
        dup.overwritten_height,
        SupplyState {
            value: Sats::INITIAL_SUBSIDY,
            utxo_count: 1,
        },
    ))
}

/// The earlier coinbase whose outputs a BIP30 duplicate overwrote.
pub fn overwritten_tx_index(tx_index: TxIndex) -> Option<TxIndex> {
    DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.tx_index == tx_index)
        .map(|dup| dup.overwritten_tx_index)
}
