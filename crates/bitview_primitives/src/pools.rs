use std::sync::OnceLock;

use brk_types::AddressBytes;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::from_str;

use crate::{Pool, PoolSlug};

#[cfg(feature = "storage")]
use vecdb::Version;

/// Increment when the matching rule changes. The list's own changes (a pool, its number, its addresses
/// or tags) count by themselves: see `Pools::version`.
#[cfg(feature = "storage")]
pub const POOL_ATTRIBUTION_VERSION: Version = Version::new(7);

/// DebitMyData's pool list as this build was made (DebitMyData-PoW/mining-pools' `pools-v2.json`):
/// read when no newer copy is given (`BITVIEW_POOLS`).
const JSON_DATA: &str = include_str!("../pools-v2.json");
/// Where a newer copy of the list is, if anywhere: a file the server's updater keeps (see
/// `deploy/debitmydata/pools.sh`), read when the server starts.
pub const POOLS_FILE_VAR: &str = "BITVIEW_POOLS";

#[derive(Deserialize)]
struct JsonPoolEntry {
    id: u16,
    name: String,
    #[serde(rename = "addresses", default)]
    addrs: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    link: String,
    /// Where the pool is run from: two letters ("US") or the country's name ("Turkey"), if known.
    #[serde(default)]
    country: Option<String>,
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// A pool's slug: its name's lowercase letters and digits, "Coin-Miners" `coinminers`.
pub fn slug_of(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn empty_pool(id: usize) -> Pool {
    Pool {
        slug: PoolSlug::from(id as u8),
        key: leak_str(format!("pool{id}")),
        name: "",
        lead: "",
        addrs: Box::new([]),
        tags: Box::new([]),
        tags_lowercase: Box::new([]),
        link: "",
        country: None,
    }
}

#[derive(Debug)]
pub struct Pools {
    pools: Vec<Pool>,
    by_addr: FxHashMap<AddressBytes, PoolSlug>,
    by_key: FxHashMap<&'static str, PoolSlug>,
    /// A digest of what attributes a block (each pool's number, name, addresses and tags): when it
    /// changes, every block is attributed again.
    digest: u32,
    /// Where the list was read from: the file given, else this build's copy.
    pub source: &'static str,
}

impl Pools {
    pub fn find_from_coinbase_tag(&self, coinbase_tag: &str) -> Option<&Pool> {
        let coinbase_tag = coinbase_tag.to_lowercase();
        self.iter().find(|pool| {
            pool.tags_lowercase
                .iter()
                .any(|pool_tag| coinbase_tag.contains(pool_tag))
        })
    }

    pub fn find_from_addr(&self, addr: &AddressBytes) -> Option<&Pool> {
        self.by_addr.get(addr).map(|&slug| self.get(slug))
    }

    pub fn unknown(&self) -> &Pool {
        &self.pools[0]
    }

    /// The table's entry for `slug`: an empty one for a reserved slug (no pool has it).
    pub fn get(&self, slug: PoolSlug) -> &Pool {
        &self.pools[usize::from(u8::from(slug))]
    }

    /// The pool written `key` (its slug), `unknown` included.
    pub fn find_slug(&self, key: &str) -> Option<PoolSlug> {
        self.by_key.get(key).copied()
    }

    /// The list's own version: a block is attributed again when it changes.
    #[cfg(feature = "storage")]
    pub fn version(&self) -> Version {
        Version::new(self.digest)
    }

    /// The pool named `slug`; none for a reserved slug.
    pub fn find(&self, slug: PoolSlug) -> Option<&Pool> {
        Some(self.get(slug)).filter(|pool| !pool.name.is_empty())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pool> + '_ {
        self.pools.iter().filter(|pool| !pool.name.is_empty())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

/// The pool list: the file `BITVIEW_POOLS` names, if it reads as one, else this build's copy. Read
/// once, when first asked.
pub fn pools() -> &'static Pools {
    static POOLS: OnceLock<Pools> = OnceLock::new();
    POOLS.get_or_init(|| {
        let given = std::env::var_os(POOLS_FILE_VAR).and_then(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let entries = from_str::<Vec<JsonPoolEntry>>(&text).ok()?;
            Some((entries, leak_str(path.to_string_lossy().into_owned())))
        });
        let (entries, source) = given.unwrap_or_else(|| {
            let entries: Vec<JsonPoolEntry> = from_str(JSON_DATA).expect("Failed to parse pools-v2.json");
            (entries, "built in")
        });
        build(entries, source)
    })
}

fn build(entries: Vec<JsonPoolEntry>, source: &'static str) -> Pools {
    let mut pools: Vec<Pool> = (0..=usize::from(u8::MAX)).map(empty_pool).collect();

    pools[0].key = "unknown";
    pools[0].name = "Unknown";
    pools[0].lead = "Blocks not matched to a known mining pool.";

    // (FNV-1a over what attributes a block: the same list, the same digest, on every server.)
    let mut digest: u32 = 0x811c_9dc5;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            digest = (digest ^ u32::from(b)).wrapping_mul(0x0100_0193);
        }
        digest = (digest ^ 0xff).wrapping_mul(0x0100_0193);
    };

    let mut taken = FxHashMap::default();
    for entry in entries {
        // (A number past the slots, 0 (no pool's), a number or a slug a pool before it has: left out.)
        let Ok(id) = u8::try_from(entry.id) else {
            continue;
        };
        let id = usize::from(id);
        let key = slug_of(&entry.name);
        if id == 0
            || !pools[id].name.is_empty()
            || key.is_empty()
            || key == "unknown"
            || taken.contains_key(key.as_str())
        {
            continue;
        }
        eat(&entry.id.to_le_bytes());
        eat(entry.name.as_bytes());
        entry.addrs.iter().for_each(|a| eat(a.as_bytes()));
        entry.tags.iter().for_each(|t| eat(t.as_bytes()));
        let key = leak_str(key);
        taken.insert(key, ());
        let tags_lowercase = entry
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let country = entry.country.as_deref().and_then(crate::country_code).map(leak_str);
        pools[id] = Pool {
            slug: PoolSlug::from(id as u8),
            key,
            lead: leak_str(format!(
                "Blocks mined by {}{}",
                entry.name,
                if entry.name.ends_with('.') { "" } else { "." }
            )),
            name: leak_str(entry.name),
            link: leak_str(entry.link),
            addrs: entry
                .addrs
                .into_iter()
                .map(leak_str)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            tags: entry
                .tags
                .into_iter()
                .map(leak_str)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            tags_lowercase,
            country,
        };
    }

    let mut by_addr = FxHashMap::default();
    let mut by_key = FxHashMap::default();
    by_key.insert("unknown", PoolSlug::Unknown);
    for pool in pools.iter().filter(|pool| !pool.name.is_empty()) {
        by_key.insert(pool.key, pool.slug);
        for addr in &pool.addrs {
            if let Ok(addr) = addr.parse() {
                by_addr.entry(addr).or_insert(pool.slug);
            }
        }
    }

    Pools {
        pools,
        by_addr,
        by_key,
        // (Kept under 2^24: a version is added to others.)
        digest: digest % (1 << 24),
        source,
    }
}
