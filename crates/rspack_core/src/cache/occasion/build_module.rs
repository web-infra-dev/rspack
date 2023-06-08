use std::{path::Path, sync::Arc};

use futures::Future;
use rspack_error::{Result, TWithDiagnosticArray};

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildResult,
};

type Storage = dyn storage::Storage<(Snapshot, TWithDiagnosticArray<BuildResult>)>;

#[derive(Debug)]
pub struct BuildModuleOccasion {
  storage: Option<Box<Storage>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl BuildModuleOccasion {
  pub fn new(storage: Option<Box<Storage>>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage,
      snapshot_manager,
    }
  }

  pub async fn use_cache<'a, G, F>(
    &self,
    module: &'a mut BoxModule,
    generator: G,
  ) -> Result<(Result<TWithDiagnosticArray<BuildResult>>, bool)>
  where
    G: Fn(&'a mut BoxModule) -> F,
    F: Future<Output = Result<TWithDiagnosticArray<BuildResult>>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return Ok((Ok(generator(module).await?), false)),
    };

    let mut need_cache = false;
    let id = module.identifier().to_owned();
    if module.as_normal_module().is_some() {
      // normal module
      // TODO cache all module type
      if let Some((snapshot, data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);
        if valid {
          return Ok((Ok(data), true));
        }
      };
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module).await?;
    if need_cache && data.inner.build_info.cacheable {
      let mut paths: Vec<&Path> = Vec::new();
      paths.extend(
        data
          .inner
          .build_info
          .file_dependencies
          .iter()
          .map(|i| i.as_path()),
      );
      paths.extend(
        data
          .inner
          .build_info
          .context_dependencies
          .iter()
          .map(|i| i.as_path()),
      );
      paths.extend(
        data
          .inner
          .build_info
          .missing_dependencies
          .iter()
          .map(|i| i.as_path()),
      );
      paths.extend(
        data
          .inner
          .build_info
          .build_dependencies
          .iter()
          .map(|i| i.as_path()),
      );

      let snapshot = self
        .snapshot_manager
        .create_snapshot(&paths, |option| &option.module)
        .await?;
      storage.set(id, (snapshot, data.clone()));
    }
    Ok((Ok(data), false))
  }
}
