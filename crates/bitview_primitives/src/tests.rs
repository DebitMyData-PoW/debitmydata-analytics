use std::str::FromStr;

use crate::PoolSlug;

/// A slug this server doesn't know reads as `Unknown` through serde, never through `FromStr`.
#[test]
fn unknown_pool_slugs() {
    let read = |json: &str| serde_json::from_str::<PoolSlug>(json).unwrap();
    assert_eq!(read("\"coinminers\"").as_str(), "coinminers");
    assert_eq!(read("\"a_pool_from_the_future\""), PoolSlug::Unknown);
    assert_eq!(
        serde_json::to_string(&PoolSlug::Unknown).unwrap(),
        "\"unknown\""
    );
    assert!(PoolSlug::from_str("a_pool_from_the_future").is_err());
}

/// A price above the compact range saturates at its top, never wraps to a small one.
#[test]
fn cents_compact_saturates() {
    use brk_types::Cents;

    use crate::CentsCompact;

    let top = CentsCompact::from(Cents::from(u64::from(u32::MAX) - 1));
    assert_eq!(
        CentsCompact::from(Cents::from(u64::from(u32::MAX) + 1)),
        top
    );
    assert_eq!(CentsCompact::from(Cents::from(10_000_000_000_u64)), top);
    assert!(!top.is_nan());
    assert_eq!(CentsCompact::from(Cents::from(12_345_u64)).inner(), 12_345);
}

/// DebitMyData's pools, from DebitMyData-PoW/mining-pools: the official pool known by the text it
/// writes in its blocks' coinbase, each pool's slug its name's letters and digits.
#[test]
fn the_debitmydata_pools() {
    let pools = crate::pools();
    assert_eq!(pools.source, "built in");
    let pool = pools.find_from_coinbase_tag("/YiiMP/ Official").unwrap();
    assert_eq!((pool.key, pool.name), ("official", "Official"));
    assert_eq!(pool.link(), "https://pool.debitmydata.io/");
    assert!(
        pools
            .find_from_coinbase_tag("/Foundry USA Pool #dropgold/")
            .is_none()
    );
    let slug = PoolSlug::from_str("coinminers").unwrap();
    assert_eq!((u8::from(slug), pools.get(slug).name), (2, "Coin-Miners"));
    assert_eq!(PoolSlug::from_str("unknown").unwrap(), PoolSlug::Unknown);
    assert!(pools.len() >= 7);
    // (A slot no pool has keeps a spelling of its own, never another pool's.)
    assert_eq!(PoolSlug::from(200).as_str(), "pool200");
}

/// The slug rule, and a country by its code or its name.
#[test]
fn pool_slugs_and_countries() {
    assert_eq!(crate::country_code("us").as_deref(), Some("US"));
    assert_eq!(crate::country_code("Turkey").as_deref(), Some("TR"));
    assert_eq!(crate::country_code("United Arab Emirates").as_deref(), Some("AE"));
    assert_eq!(crate::country_code("Atlantis"), None);
    assert_eq!(crate::slug_of("Coin-Miners"), "coinminers");
    assert_eq!(crate::slug_of("KriptoKyng"), "kriptokyng");
    assert_eq!(crate::slug_of("Mined by pep!"), "minedbypep");
}

/// Five bytes hold values below 2^40 - 1 as they are, and all ones stand for `u64::MAX`, the
/// sentinel `TxInIndex::UNSPENT` is stored as.
#[test]
fn index40_round_trips_and_keeps_its_sentinel() {
    use crate::{Index40, TxInIndex};

    for value in [0, 1, 255, 1 << 32, (1 << 40) - 2] {
        assert_eq!(Index40::<u64>::new(value).get(), value);
    }
    let unspent = Index40::new(TxInIndex::UNSPENT);
    assert_eq!(unspent.get(), TxInIndex::UNSPENT);
    assert_eq!(Index40::<u64>::new(u64::MAX).get(), u64::MAX);
    #[cfg(feature = "storage")]
    assert_eq!(vecdb::Bytes::to_bytes(&unspent), [0xff; 5]);
}
