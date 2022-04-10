use std::hash::Hash;

use smol_str::SmolStr;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolvedId {
    pub id: SmolStr,
    pub external: bool,
}

impl ResolvedId {
    pub fn new<T: Into<SmolStr>>(id: T, external: bool) -> Self {
        Self {
            id: id.into(),
            external,
            // module_side_effects: false,
        }
    }
}

pub type ResolveIdResult = Option<ResolvedId>;
