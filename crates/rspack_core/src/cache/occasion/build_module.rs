use std::{path::Path, sync::Arc};

use futures::Future;
use rspack_error::{Diagnostic, Result, TWithDiagnosticArray};
use rspack_identifier::Identifier;
use rspack_sources::BoxSource;

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildResult, NormalModuleSource,
};

type Storage = dyn storage::Storage<(
  Snapshot,
  Option<NormalModuleSource>,
  TWithDiagnosticArray<BuildResult>,
)>;

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

  pub fn remove_cache(&self, id: &Identifier) {
    if let Some(s) = self.storage.as_ref() {
      s.remove(id);
    }
  }

  pub async fn use_cache<'m, G, F>(
    &self,
    module: &'m mut BoxModule,
    generator: G,
  ) -> Result<(Result<TWithDiagnosticArray<BuildResult>>, bool)>
  where
    G: Fn(&'m mut BoxModule) -> (F),
    F: Future<Output = Result<(TWithDiagnosticArray<BuildResult>, &'m mut BoxModule)>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return Ok((Ok(generator(module).await?.0), false)),
    };

    let mut need_cache = false;
    let id = module.identifier().to_owned();
    if module.as_normal_module().is_some() {
      // normal module
      // TODO cache all module type
      if let Some((snapshot, source, data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);
        if valid {
          if let Some(module) = module.as_normal_module_mut() {
            *module.source_mut() = source.unwrap();
          }
          return Ok((Ok(data), true));
        }
      };
      need_cache = true;
    }

    // run generator and save to cache
    let (data, module) = generator(module).await?;
    let source = module.as_normal_module().unwrap().source().clone();
    std::mem::drop(source);
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
      storage.set(id, (snapshot, None, data.clone()));
    }
    Ok((Ok(data), false))
  }
}
