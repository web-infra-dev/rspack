use std::{
  hash::{Hash, Hasher},
  ops::Sub,
  path::PathBuf,
};

use rayon::prelude::*;
use rspack_error::Result;
use rspack_fs::AsyncWritableFileSystem;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rspack_sources::{RawSource, SourceExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  fast_set, AssetInfo, Chunk, ChunkKind, Compilation, CompilationAsset, Compiler, ModuleIdentifier,
  PathData, RenderManifestArgs, RuntimeSpec, SetupMakeParam,
};

#[derive(Default)]
struct HotUpdateContent {
  runtime: RuntimeSpec,
  updated_chunk_ids: HashSet<String>,
  removed_chunk_ids: HashSet<String>,
  _removed_modules: IdentifierSet,
}

impl HotUpdateContent {
  fn new(runtime: RuntimeSpec) -> Self {
    Self {
      runtime,
      ..Default::default()
    }
  }
}

impl<T> Compiler<T>
where
  T: AsyncWritableFileSystem + Send + Sync,
{
  // TODO: remove this function when we had `record` in compiler.
  pub async fn rebuild(
    &mut self,
    changed_files: std::collections::HashSet<String>,
    removed_files: std::collections::HashSet<String>,
  ) -> Result<()> {
    assert!(!changed_files.is_empty() || !removed_files.is_empty());
    let old = self.compilation.get_stats();
    let old_hash = self.compilation.hash.clone();
    fn collect_changed_modules(
      compilation: &Compilation,
    ) -> (IdentifierMap<(u64, String)>, IdentifierMap<String>) {
      let modules_map = compilation
        .chunk_graph
        .chunk_graph_module_by_module_identifier
        .par_iter()
        .filter_map(|(identifier, cgm)| {
          let module_hash = compilation.module_graph.get_module_hash(identifier);
          let cid = cgm.id.as_deref();
          if let Some(module_hash) = module_hash && let Some(cid) = cid {
              Some((
                  *identifier,
                  (
                      module_hash,
                      cid.to_string(),
                  ),
              ))
          } else {
              None
          }
        })
        .collect::<IdentifierMap<_>>();

      let old_runtime_modules = compilation
        .runtime_modules
        .iter()
        .map(|(identifier, module)| {
          (
            *identifier,
            module.generate(compilation).source().to_string(),
          )
        })
        .collect();

      (modules_map, old_runtime_modules)
    }

    let (old_all_modules, old_runtime_modules) = collect_changed_modules(old.compilation);
    // TODO: should use `records`

    let mut all_old_runtime: RuntimeSpec = Default::default();
    for entrypoint_ukey in old.compilation.entrypoints.values() {
      if let Some(runtime) = old
        .compilation
        .chunk_group_by_ukey
        .get(entrypoint_ukey)
        .map(|entrypoint| entrypoint.runtime.clone())
      {
        all_old_runtime.extend(runtime);
      }
    }

    let mut hot_update_main_content_by_runtime = all_old_runtime
      .iter()
      .map(|runtime| {
        (
          runtime.to_string(),
          HotUpdateContent::new(HashSet::from_iter([runtime.clone()])),
        )
      })
      .collect::<HashMap<String, HotUpdateContent>>();

    let mut old_chunks: Vec<(String, IdentifierSet, RuntimeSpec)> = vec![];
    for (ukey, chunk) in old.compilation.chunk_by_ukey.iter() {
      let modules = old
        .compilation
        .chunk_graph
        .get_chunk_graph_chunk(ukey)
        .modules
        .clone();
      old_chunks.push((
        chunk.expect_id().to_string(),
        modules,
        chunk.runtime.clone(),
      ));
    }

    // build without stats
    {
      let mut modified_files = HashSet::default();
      modified_files.extend(changed_files.iter().map(PathBuf::from));
      modified_files.extend(removed_files.iter().map(PathBuf::from));

      self.cache.end_idle();
      self
        .cache
        .set_modified_files(modified_files.iter().cloned().collect::<Vec<_>>());
      self
        .plugin_driver
        .read()
        .await
        .resolver_factory
        .clear_entries();

      let mut new_compilation = Compilation::new(
        // TODO: use Arc<T> instead
        self.options.clone(),
        self.options.entry.clone(),
        Default::default(),
        self.plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.cache.clone(),
      );

      let is_incremental_rebuild = self.options.is_incremental_rebuild();
      if is_incremental_rebuild {
        // copy field from old compilation
        // make stage used
        new_compilation.module_graph = std::mem::take(&mut self.compilation.module_graph);
        new_compilation.make_failed_dependencies =
          std::mem::take(&mut self.compilation.make_failed_dependencies);
        new_compilation.make_failed_module =
          std::mem::take(&mut self.compilation.make_failed_module);
        new_compilation.entry_dependencies =
          std::mem::take(&mut self.compilation.entry_dependencies);
        new_compilation.lazy_visit_modules =
          std::mem::take(&mut self.compilation.lazy_visit_modules);
        new_compilation.file_dependencies = std::mem::take(&mut self.compilation.file_dependencies);
        new_compilation.context_dependencies =
          std::mem::take(&mut self.compilation.context_dependencies);
        new_compilation.missing_dependencies =
          std::mem::take(&mut self.compilation.missing_dependencies);
        new_compilation.build_dependencies =
          std::mem::take(&mut self.compilation.build_dependencies);
        // tree shaking usage start
        new_compilation.optimize_analyze_result_map =
          std::mem::take(&mut self.compilation.optimize_analyze_result_map);
        new_compilation.entry_module_identifiers =
          std::mem::take(&mut self.compilation.entry_module_identifiers);
        new_compilation.bailout_module_identifiers =
          std::mem::take(&mut self.compilation.bailout_module_identifiers);
        // tree shaking usage end

        // seal stage used
        new_compilation.code_splitting_cache =
          std::mem::take(&mut self.compilation.code_splitting_cache);
      } else {
        new_compilation.setup_entry_dependencies();
      }

      fast_set(&mut self.compilation, new_compilation);

      self.compilation.lazy_visit_modules = changed_files.clone();

      // Fake this compilation as *currently* rebuilding does not create a new compilation
      self
        .plugin_driver
        .write()
        .await
        .this_compilation(&mut self.compilation)
        .await?;

      self
        .plugin_driver
        .write()
        .await
        .compilation(&mut self.compilation)
        .await?;

      let setup_make_params = if is_incremental_rebuild {
        SetupMakeParam::ModifiedFiles(modified_files)
      } else {
        let deps = self
          .compilation
          .entry_dependencies
          .iter()
          .flat_map(|(_, deps)| {
            deps
              .clone()
              .into_iter()
              .map(|d| (d, None))
              .collect::<Vec<_>>()
          })
          .collect::<HashSet<_>>();
        SetupMakeParam::ForceBuildDeps(deps)
      };
      self.compile(setup_make_params).await?;
      self.cache.begin_idle();
    }

    // ----
    if hot_update_main_content_by_runtime.is_empty() {
      self.compile_done().await?;
      return Ok(());
    }

    let (now_all_modules, now_runtime_modules) = collect_changed_modules(&self.compilation);

    let mut updated_modules: IdentifierSet = Default::default();
    let mut updated_runtime_modules: IdentifierSet = Default::default();
    let mut completely_removed_modules: HashSet<String> = Default::default();

    for (old_uri, (old_hash, old_module_id)) in &old_all_modules {
      if let Some((now_hash, _)) = now_all_modules.get(old_uri) {
        // updated
        if now_hash != old_hash {
          updated_modules.insert(*old_uri);
        }
      } else {
        // deleted
        completely_removed_modules.insert(old_module_id.to_string());
      }
    }
    for identifier in now_all_modules.keys() {
      if !old_all_modules.contains_key(identifier) {
        // added
        updated_modules.insert(*identifier);
      }
    }

    // println!(
    //   "updated_modules: {:?}\n, remove modules {:?}",
    //   updated_modules, completely_removed_modules
    // );

    for (identifier, old_runtime_module_content) in &old_runtime_modules {
      if let Some(new_runtime_module_content) = now_runtime_modules.get(identifier) {
        // updated
        if new_runtime_module_content != old_runtime_module_content {
          updated_runtime_modules.insert(*identifier);
        }
      }
    }
    for identifier in now_runtime_modules.keys() {
      if !old_runtime_modules.contains_key(identifier) {
        // added
        updated_runtime_modules.insert(*identifier);
      }
    }

    // TODO: hash
    // if old.hash == now.hash { return  } else { // xxxx}

    for (chunk_id, _old_chunk_modules, old_runtime) in &old_chunks {
      let mut new_modules = vec![];
      let mut new_runtime_modules = vec![];
      let mut chunk_id = chunk_id.to_string();
      let mut new_runtime = all_old_runtime.clone();
      let mut removed_from_runtime = all_old_runtime.clone();
      let current_chunk = self
        .compilation
        .chunk_by_ukey
        .iter()
        .find(|(_, chunk)| chunk.expect_id().eq(&chunk_id))
        .map(|(_, chunk)| chunk);

      if let Some(current_chunk) = current_chunk {
        chunk_id = current_chunk.expect_id().to_string();
        new_runtime = Default::default();
        // intersectRuntime
        for old_runtime in &all_old_runtime {
          if current_chunk.runtime.contains(old_runtime) {
            new_runtime.insert(old_runtime.clone());
          }
        }
        // ------
        if new_runtime.is_empty() {
          continue;
        }

        new_modules = self
          .compilation
          .chunk_graph
          .get_chunk_graph_chunk(&current_chunk.ukey)
          .modules
          .iter()
          .filter_map(|module| updated_modules.contains(module).then_some(*module))
          .collect::<Vec<_>>();

        new_runtime_modules = self
          .compilation
          .chunk_graph
          .get_chunk_runtime_modules_in_order(&current_chunk.ukey)
          .iter()
          .filter_map(|module| {
            updated_runtime_modules
              .contains(module)
              .then(|| ModuleIdentifier::from(module.as_str()))
          })
          .collect::<Vec<_>>();

        // subtractRuntime
        removed_from_runtime = removed_from_runtime.sub(&new_runtime);
      } else {
        removed_from_runtime = old_runtime.clone();
        // new_runtime = old_runtime.clone();
      }

      for removed in removed_from_runtime {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(removed.as_ref()) {
          info.removed_chunk_ids.insert(chunk_id.to_string());
        }
        // TODO:
        // for (const module of remainingModules) {}
      }

      if !new_modules.is_empty() || !new_runtime_modules.is_empty() {
        let mut hot_update_chunk = Chunk::new(
          Some(chunk_id.to_string()),
          Some(chunk_id.to_string()),
          ChunkKind::HotUpdate,
        );
        hot_update_chunk.runtime = new_runtime.clone();
        let ukey = hot_update_chunk.ukey;
        if let Some(current_chunk) = current_chunk {
          current_chunk
            .groups
            .iter()
            .for_each(|group| hot_update_chunk.add_group(*group))
        }

        for module_identifier in new_modules.iter() {
          if let Some(module) = self
            .compilation
            .module_graph
            .module_by_identifier(module_identifier)
          {
            module.hash(&mut hot_update_chunk.hash);
          }
        }
        let hash = format!("{:016x}", hot_update_chunk.hash.finish());
        hot_update_chunk
          .content_hash
          .insert(crate::SourceType::JavaScript, hash.clone());
        hot_update_chunk
          .content_hash
          .insert(crate::SourceType::Css, hash);

        self.compilation.chunk_by_ukey.add(hot_update_chunk);
        self.compilation.chunk_graph.add_chunk(ukey);

        for module_identifier in new_modules.iter() {
          self
            .compilation
            .chunk_graph
            .connect_chunk_and_module(ukey, *module_identifier);
        }

        for runtime_module in new_runtime_modules {
          self
            .compilation
            .chunk_graph
            .connect_chunk_and_runtime_module(ukey, runtime_module);
        }

        let render_manifest = self
          .compilation
          .plugin_driver
          .read()
          .await
          .render_manifest(RenderManifestArgs {
            compilation: &self.compilation,
            chunk_ukey: ukey,
          })
          .await
          .expect("render_manifest failed in rebuild");

        for entry in render_manifest {
          let asset = CompilationAsset::new(
            Some(entry.source),
            entry.info.with_hot_module_replacement(true),
          );

          let chunk = self
            .compilation
            .chunk_by_ukey
            .get(&ukey)
            .expect("should have update chunk");
          let filename = self.compilation.get_path(
            &self.compilation.options.output.hot_update_chunk_filename,
            PathData::default().chunk(chunk).hash(&old_hash),
          );
          self.compilation.emit_asset(filename, asset);
        }

        new_runtime.iter().for_each(|runtime| {
          if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime.as_ref()) {
            info.updated_chunk_ids.insert(chunk_id.to_string());
          }
        });
      }
    }

    let completely_removed_modules_array: Vec<String> =
      completely_removed_modules.into_iter().collect();

    for (_, content) in hot_update_main_content_by_runtime {
      let c: Vec<String> = content.updated_chunk_ids.into_iter().collect();
      let r: Vec<String> = content.removed_chunk_ids.into_iter().collect();
      let m: Vec<String> = completely_removed_modules_array
        .iter()
        .map(|x| x.to_owned())
        .collect();
      let filename = self.compilation.get_path(
        &self.compilation.options.output.hot_update_main_filename,
        PathData::default()
          .runtime(&content.runtime)
          .hash(&old_hash),
      );
      self.compilation.emit_asset(
        filename,
        CompilationAsset::new(
          Some(
            RawSource::Source(
              serde_json::json!({
                "c": c,
                "r": r,
                "m": m,
              })
              .to_string(),
            )
            .boxed(),
          ),
          AssetInfo::default().with_hot_module_replacement(true),
        ),
      );
    }

    self.compile_done().await?;

    Ok(())
  }
}
