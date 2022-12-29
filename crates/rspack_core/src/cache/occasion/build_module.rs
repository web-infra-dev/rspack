use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildResult,
};
use futures::Future;
use rspack_error::{Result, TWithDiagnosticArray};
use std::sync::Arc;

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
  ) -> Result<TWithDiagnosticArray<BuildResult>>
  where
    G: Fn(&'a mut BoxModule) -> F,
    F: Future<Output = Result<TWithDiagnosticArray<BuildResult>>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator(module).await,
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
          return Ok(data);
        }
      };
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module).await?;
    if need_cache && data.inner.cacheable {
      let mut paths: Vec<&str> = data
        .inner
        .build_dependencies
        .iter()
        .map(|i| i.as_str())
        .collect();
      paths.push(&id);

      let snapshot = self
        .snapshot_manager
        // TODO replace id with source file path or just cache normal module
        .create_snapshot(paths, |option| &option.module)
        .await?;
      storage.set(id, (snapshot, data.clone()));
    }
    Ok(data)
  }
}
