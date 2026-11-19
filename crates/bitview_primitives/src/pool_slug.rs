use std::{borrow::Cow, fmt};

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Error, Formattable, Result};

/// A mining pool's identifier: its number in the pool list (`pools-v2.json`'s `id`, 1 to 255; 0 is
/// the blocks no pool claims), written as its slug in URLs, JSON and series ids.
///
/// The slugs are DebitMyData's pools', read with the list when the server starts (see `pools()`):
/// each pool's name in lowercase letters and digits, "Coin-Miners" `coinminers`. A pool added to
/// the list gets its number's slot, its series and its slug at the next start.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoolSlug(u8);

impl PoolSlug {
    /// The blocks no pool claims.
    #[allow(non_upper_case_globals)]
    pub const Unknown: Self = Self(0);

    /// Its spelling: the pool's slug, `unknown` for no pool, and for a number the list doesn't
    /// name, `pool` and the number (never written by the server: such a slot has no series).
    pub fn as_str(self) -> &'static str {
        crate::pools().get(self).key
    }

    /// Its slot, 0 to 255.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl Serialize for PoolSlug {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// A slug this server doesn't know reads as `Unknown`, so a client older than the server still
/// decodes the responses naming a newer pool.
impl<'de> Deserialize<'de> for PoolSlug {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let slug = Cow::<'de, str>::deserialize(deserializer)?;
        Ok(slug.parse().unwrap_or(Self::Unknown))
    }
}

impl std::str::FromStr for PoolSlug {
    type Err = ();

    /// The pool written `slug`; an unknown slug is an error (serde reads it as `Unknown`).
    fn from_str(slug: &str) -> std::result::Result<Self, ()> {
        crate::pools().find_slug(slug).ok_or(())
    }
}

/// The slugs of the pools this server knows, as an enumeration: the API's document and the
/// clients made from it list them.
impl JsonSchema for PoolSlug {
    fn schema_name() -> Cow<'static, str> {
        "PoolSlug".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let slugs = std::iter::once(Self::Unknown.as_str())
            .chain(crate::pools().iter().map(|pool| pool.key).filter(|key| *key != "unknown"))
            .collect::<Vec<_>>();
        schemars::json_schema!({
            "description": "A mining pool's identifier, as written in URLs: its name in lowercase letters and digits.",
            "type": "string",
            "enum": slugs,
        })
    }
}

impl AsRef<str> for PoolSlug {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PoolSlug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "storage")]
impl Formattable for PoolSlug {
    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_str().as_bytes());
    }

    #[inline]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

#[cfg(feature = "storage")]
// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for PoolSlug {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [self.0]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        };
        Ok(Self(bytes[0]))
    }
}

impl From<u8> for PoolSlug {
    #[inline]
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<PoolSlug> for u8 {
    #[inline]
    fn from(val: PoolSlug) -> u8 {
        val.0
    }
}
