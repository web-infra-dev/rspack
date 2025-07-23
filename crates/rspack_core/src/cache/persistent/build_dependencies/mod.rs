mod helper;

use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, AssertUtf8};
use rspack_regex::RspackRegex;
use rustc_hash::FxHashSet as HashSet;

use self::helper::Helper;
use super::{
  snapshot::{PathMatcher, Snapshot, SnapshotOptions},
  storage::Storage,
};

const SCOPE: &str = "build_dependencies";

pub type BuildDepsOptions = Vec<PathBuf>;

#[derive(Debug)]
pub struct BuildDeps {
  added: HashSet<ArcPath>,
  pending: HashSet<ArcPath>,
  snapshot: Snapshot,
  storage: Arc<dyn Storage>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl BuildDeps {
  pub fn new(
    options: &BuildDepsOptions,
    fs: Arc<dyn ReadableFileSystem>,
    storage: Arc<dyn Storage>,
  ) -> Self {
    Self {
      added: Default::default(),
      pending: options.iter().map(|v| ArcPath::from(v.as_path())).collect(),
      snapshot: Snapshot::new_with_scope(
        SCOPE,
        SnapshotOptions::new(
          Default::default(),
          Default::default(),
          vec![PathMatcher::Regexp(
            RspackRegex::new("[/\\\\]node_modules[/\\\\]").expect("should generate regex"),
          )],
        ),
        fs.clone(),
        storage.clone(),
      ),
      storage,
      fs,
    }
  }

  pub async fn add(&mut self, data: impl Iterator<Item = ArcPath>) -> Vec<String> {
    let mut helper = Helper::new(self.fs.clone());
    let mut new_deps = HashSet::default();
    let mut queue = VecDeque::new();
    queue.extend(std::mem::take(&mut self.pending));
    queue.extend(data);
    loop {
      let Some(current) = queue.pop_front() else {
        break;
      };
      if !self.added.insert(current.clone()) {
        continue;
      }
      new_deps.insert(current.clone());
      if let Some(childs) = helper.resolve(current.assert_utf8()).await {
        queue.extend(childs.iter().map(|item| item.as_path().into()));
      }
    }

    self.snapshot.add(new_deps.into_iter()).await;
    helper.into_warnings()
  }

  pub async fn validate(&mut self) -> Result<()> {
    let (_, modified_files, removed_files, no_changed_files) =
      self.snapshot.calc_modified_paths().await?;

    if !modified_files.is_empty() || !removed_files.is_empty() {
      self.storage.reset().await;

      tracing::info!(
        "BuildDependencies: cache invalidate by modified_files {modified_files:?} and removed_files {removed_files:?}"
      );
      return Ok(());
    }
    self.added = no_changed_files;
    Ok(())
  }
}
