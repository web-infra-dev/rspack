use std::sync::Arc;

use rspack_paths::InternedPath;

use crate::cache::persistent::snapshot::{
  SnapshotOptions, SnapshotStrategyOptions, Strategy, StrategyHelper,
};

#[derive(Debug, Clone, Copy)]
pub(super) enum SnapshotScope {
  File,
  Context,
  Missing,
  Build,
}

pub(super) async fn calc_strategy(
  options: &Arc<SnapshotOptions>,
  helper: &StrategyHelper,
  path: &InternedPath,
  scope: SnapshotScope,
) -> Option<Strategy> {
  let path_str = path.to_string_lossy();
  if options.is_immutable_path(&path_str) {
    return None;
  }
  if options.is_managed_path(&path_str)
    && let Some(strategy) = helper.package_version(path).await
  {
    return Some(strategy);
  }
  Some(match scope {
    SnapshotScope::File => {
      helper
        .file_strategy(path, options.dependencies_strategy())
        .await
    }
    SnapshotScope::Context => {
      helper
        .dir_strategy(path, options.context_dependencies_strategy())
        .await
    }
    SnapshotScope::Missing => Strategy::Missing,
    SnapshotScope::Build => {
      helper
        .dir_strategy(path, SnapshotStrategyOptions::hash())
        .await
    }
  })
}
