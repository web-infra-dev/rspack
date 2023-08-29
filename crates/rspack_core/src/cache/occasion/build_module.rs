use std::{collections::HashMap, path::Path, sync::Arc};

use futures::Future;
use rkyv::AlignedVec;
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_identifier::Identifier;

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildExtraDataType, BuildResult, NormalModuleSource,
};

type Storage = dyn storage::Storage<(
  Snapshot,
  Option<NormalModuleSource>,
  TWithDiagnosticArray<BuildResult>,
  HashMap<BuildExtraDataType, AlignedVec>,
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

  pub async fn use_cache<'a, G, F>(
    &self,
    module: &'a mut BoxModule,
    generator: G,
  ) -> Result<(Result<TWithDiagnosticArray<BuildResult>>, bool)>
  where
    G: Fn(&'a mut BoxModule) -> F,
    F: Future<Output = Result<(TWithDiagnosticArray<BuildResult>, &'a mut BoxModule)>>,
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
      if let Some((snapshot, source, data, extra_data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);
        if valid {
          if let Some(module) = module.as_normal_module_mut() {
            if let Some(module_source) = source {
              *module.source_mut() = module_source;
              module.parser_and_generator_mut().resume(&extra_data);
            }
          }
          return Ok((Ok(data), true));
        }
      };
      need_cache = true;
    }

    // run generator and save to cache
    let (data, module) = generator(module).await?;
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
      let mut extra_data = HashMap::new();
      module
        .as_normal_module()
        .unwrap()
        .parser_and_generator()
        .store(&mut extra_data);
      storage.set(
        id,
        (
          snapshot,
          Some(module.as_normal_module().unwrap().source().clone()),
          data.clone(),
          extra_data,
        ),
      );
    }
    Ok((Ok(data), false))
  }
}
