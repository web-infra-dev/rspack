use std::{collections::HashMap, path::Path, sync::Arc};

use futures::Future;
use rkyv::AlignedVec;
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_identifier::{Identifiable, Identifier};

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildExtraDataType, BuildResult, DependencyTemplate, ModuleDependency,
  NormalModuleSource,
};

type Storage = dyn storage::Storage<(
  // file system info, None when not cacheable
  Option<Snapshot>,
  // build result
  TWithDiagnosticArray<BuildResult>,
  // source
  Option<NormalModuleSource>,
  // code generation_dependencies
  Option<Vec<Box<dyn ModuleDependency>>>,
  // presentational_dependencies
  Option<Vec<Box<dyn DependencyTemplate>>>,
  // parser and generator data
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
    let mut last_build_result = None;
    let id = module.identifier().to_owned();
    if module.as_normal_module().is_some() {
      // normal module
      // TODO cache all module type
      if let Some((
        snapshot,
        data,
        source,
        code_generation_dependencies,
        presentational_dependencies,
        extra_data,
      )) = storage.get(&id)
      {
        let valid = if let Some(snapshot) = snapshot {
          self
            .snapshot_manager
            .check_snapshot_valid(&snapshot)
            .await
            .unwrap_or(false)
        } else {
          false
        };
        if valid {
          if let Some(module) = module.as_normal_module_mut() {
            if let Some(module_source) = source {
              *module.source_mut() = module_source.clone();
            }
            *module.code_generation_dependencies_mut() = code_generation_dependencies;
            *module.presentational_dependencies_mut() = presentational_dependencies;
            module.parser_and_generator_mut().resume(&extra_data);
          }
          return Ok((Ok(data), true));
        } else {
          last_build_result = Some(data.inner);
        }
      };
      need_cache = true;
    }

    // run generator and save to cache
    let (mut data, module) = generator(module).await?;

    if need_cache {
      let module = module.as_normal_module().unwrap();
      // only resume the build_meta to make sure other modules will not be affected
      if matches!(module.source(), NormalModuleSource::BuiltFailed(_)) && let Some(last_result) = last_build_result {
        data.inner.build_meta = last_result.build_meta;
        return Ok((Ok(data), false));
      }

      if data.inner.build_info.cacheable {
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
        module.parser_and_generator().store(&mut extra_data);
        storage.set(
          id,
          (
            Some(snapshot),
            data.clone(),
            Some(module.source().clone()),
            module.code_generation_dependencies().clone(),
            module.presentational_dependencies().clone(),
            extra_data,
          ),
        );
      } else if matches!(module.source(), NormalModuleSource::BuiltSucceed(_)) {
        storage.set(id, (None, data.clone(), None, None, None, HashMap::new()));
      }
    }
    Ok((Ok(data), false))
  }
}
