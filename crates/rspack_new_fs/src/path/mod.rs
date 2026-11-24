mod abs_path;
mod cacheable;
pub(crate) mod normalize;
mod project_path;

use std::{
  collections::{HashMap, HashSet},
  hash::BuildHasherDefault,
};

pub use ustr::IdentityHasher;

pub use self::{
  abs_path::AbsPath,
  cacheable::AsProjectPath,
  project_path::{ProjectPath, ProjectPathParts},
};

pub type ProjectPathMap<V> = HashMap<ProjectPath, V, BuildHasherDefault<IdentityHasher>>;
pub type ProjectPathSet = HashSet<ProjectPath, BuildHasherDefault<IdentityHasher>>;

pub type AbsPathMap<V> = HashMap<AbsPath, V, BuildHasherDefault<IdentityHasher>>;
pub type AbsPathSet = HashSet<AbsPath, BuildHasherDefault<IdentityHasher>>;
