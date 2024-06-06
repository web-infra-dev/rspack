use std::hash::BuildHasherDefault;

use dashmap::{DashMap, DashSet};
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;

pub type BuildFxHasher = BuildHasherDefault<FxHasher>;
pub type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxDashSet<V> = DashSet<V, BuildHasherDefault<FxHasher>>;
pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;
