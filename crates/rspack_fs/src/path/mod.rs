mod abs_or_rel_path;
mod abs_path;
mod cacheable;
pub(crate) mod normalize;
mod path_list;
mod project_path;
mod rel_path;

use std::{
  collections::{HashMap, HashSet},
  hash::BuildHasherDefault,
};

use dashmap::{DashMap, DashSet};
use indexmap::{IndexMap, IndexSet};
pub use ustr::IdentityHasher;

pub use self::{
  abs_or_rel_path::AbsOrRelPath,
  abs_path::{AbsPath, AbsPathAncestors},
  cacheable::AsProjectPath,
  path_list::AbsPathList,
  project_path::{ProjectPath, ProjectPathParts},
  rel_path::RelPath,
};

pub type ProjectPathMap<V> = HashMap<ProjectPath, V, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathSet = HashSet<ProjectPath, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathDashMap<V> = DashMap<ProjectPath, V, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathDashSet = DashSet<ProjectPath, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathIndexMap<V> = IndexMap<ProjectPath, V, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathIndexSet = IndexSet<ProjectPath, BuildHasherDefault<IdentityHasher>>;

pub type AbsPathMap<V> = HashMap<AbsPath, V, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathSet = HashSet<AbsPath, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathDashMap<V> = DashMap<AbsPath, V, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathDashSet = DashSet<AbsPath, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathIndexMap<V> = IndexMap<AbsPath, V, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathIndexSet = IndexSet<AbsPath, BuildHasherDefault<IdentityHasher>>;
