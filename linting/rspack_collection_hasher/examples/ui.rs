use std::{
  collections::{HashMap, HashSet},
  hash::BuildHasherDefault,
};

use dashmap::{DashMap, DashSet};
use hashlink::{LinkedHashMap, LinkedHashSet};
use indexmap::{IndexMap, IndexSet};
use rspack_collections::Identifier;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use ustr::{Ustr, UstrMap, UstrSet};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;
type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;
type FxDashSet<K> = DashSet<K, BuildHasherDefault<FxHasher>>;
type FxLinkedHashMap<K, V> = LinkedHashMap<K, V, BuildHasherDefault<FxHasher>>;
type FxLinkedHashSet<K> = LinkedHashSet<K, BuildHasherDefault<FxHasher>>;

type BadStdMap = HashMap<String, usize>;
type BadStdSet = HashSet<String>;
type BadIndexMap = IndexMap<String, usize>;
type BadIndexSet = IndexSet<String>;
type BadDashMap = DashMap<String, usize>;
type BadDashSet = DashSet<String>;
type BadLinkedHashMap = LinkedHashMap<String, usize>;
type BadLinkedHashSet = LinkedHashSet<String>;
type BadUstrFxMap = FxHashMap<Ustr, usize>;
type BadUstrFxSet = FxHashSet<Ustr>;
type BadIdentifierFxMap = FxHashMap<Identifier, usize>;
type BadIdentifierFxSet = FxHashSet<Identifier>;
type BadIdentifierFxIndexMap = FxIndexMap<Identifier, usize>;
type BadIdentifierFxIndexSet = FxIndexSet<Identifier>;
type BadIdentifierFxDashMap = FxDashMap<Identifier, usize>;
type BadIdentifierFxDashSet = FxDashSet<Identifier>;
type BadIdentifierFxLinkedHashMap = FxLinkedHashMap<Identifier, usize>;
type BadIdentifierFxLinkedHashSet = FxLinkedHashSet<Identifier>;

fn main() {
  let _: Option<BadStdMap> = None;
  let _: Option<BadStdSet> = None;
  let _: Option<BadIndexMap> = None;
  let _: Option<BadIndexSet> = None;
  let _: Option<BadDashMap> = None;
  let _: Option<BadDashSet> = None;
  let _: Option<BadLinkedHashMap> = None;
  let _: Option<BadLinkedHashSet> = None;
  let _: Option<BadUstrFxMap> = None;
  let _: Option<BadUstrFxSet> = None;
  let _: Option<BadIdentifierFxMap> = None;
  let _: Option<BadIdentifierFxSet> = None;
  let _: Option<BadIdentifierFxIndexMap> = None;
  let _: Option<BadIdentifierFxIndexSet> = None;
  let _: Option<BadIdentifierFxDashMap> = None;
  let _: Option<BadIdentifierFxDashSet> = None;
  let _: Option<BadIdentifierFxLinkedHashMap> = None;
  let _: Option<BadIdentifierFxLinkedHashSet> = None;
}
