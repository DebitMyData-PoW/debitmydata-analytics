use std::borrow::Cow;

use bitview_primitives::{Pool, PoolSlug};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolInfo {
    #[schemars(example = &"Foundry USA")]
    pub name: Cow<'static, str>,

    /// URL-friendly pool identifier
    pub slug: PoolSlug,

    /// Zero-based pool ID, matching mempool.space's `unique_id`
    #[schemars(example = 44)]
    pub unique_id: u8,

    /// The pool's website, if the pool list has one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<Cow<'static, str>>,

    /// Where the pool is run from (ISO 3166-1 alpha-2, as "US"), if the pool list says
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(example = &"US")]
    pub country: Option<Cow<'static, str>>,
}

impl From<&'static Pool> for PoolInfo {
    fn from(pool: &'static Pool) -> Self {
        Self {
            name: Cow::Borrowed(pool.name),
            slug: pool.slug,
            unique_id: pool.mempool_unique_id(),
            link: Some(pool.link()).filter(|link| !link.is_empty()).map(Cow::Borrowed),
            country: pool.country().map(Cow::Borrowed),
        }
    }
}
