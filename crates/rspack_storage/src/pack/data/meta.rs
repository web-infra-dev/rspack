use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::current_time;
use rustc_hash::FxHashSet as HashSet;

use super::options::PackOptions;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PackFileMeta {
  pub hash: String,
  pub name: String,
  pub size: usize,
  pub wrote: bool,
  pub generation: usize,
}

#[derive(Debug, Default, Clone)]
pub enum RootMetaFrom {
  #[default]
  New,
  File,
}

#[derive(Debug, Default, Clone)]
pub struct RootMeta {
  pub expire_time: u64,
  pub scopes: HashSet<String>,
  pub from: RootMetaFrom,
}

impl RootMeta {
  pub fn new(scopes: HashSet<String>, expire: u64) -> Self {
    Self {
      scopes,
      expire_time: current_time() + expire,
      from: RootMetaFrom::New,
    }
  }
  pub fn get_path(dir: &Utf8Path) -> Utf8PathBuf {
    dir.join("storage_meta")
  }
}

#[derive(Debug, Default)]
pub struct ScopeMeta {
  pub path: Utf8PathBuf,
  pub bucket_size: usize,
  pub pack_size: usize,
  pub generation: usize,
  pub packs: Vec<Vec<PackFileMeta>>,
}

impl ScopeMeta {
  pub fn new(dir: &Utf8Path, options: &PackOptions) -> Self {
    let mut packs = vec![];
    for _ in 0..options.bucket_size {
      packs.push(vec![]);
    }
    Self {
      path: Self::get_path(dir),
      bucket_size: options.bucket_size,
      pack_size: options.pack_size,
      generation: 0,
      packs,
    }
  }

  pub fn get_path(dir: &Utf8Path) -> Utf8PathBuf {
    dir.join("scope_meta")
  }
}
