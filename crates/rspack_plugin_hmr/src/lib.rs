mod hot_module_replacement;

use std::hash::Hash;

use async_trait::async_trait;
use hot_module_replacement::HotModuleReplacementRuntimeModule;
use rspack_core::{
  collect_changed_modules,
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Chunk, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAsset, CompilationParams,
  CompilationProcessAssets, CompilationRecords, CompilerCompilation, CompilerContext,
  CompilerOptions, DependencyType, LoaderContext, NormalModuleLoader, PathData, Plugin,
  PluginContext, RuntimeGlobals, RuntimeModuleExt, RuntimeSpec, SourceType,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_identifier::IdentifierSet;
use rspack_util::infallible::ResultInfallibleExt as _;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

#[plugin]
#[derive(Debug, Default)]
pub struct HotModuleReplacementPlugin;

#[plugin_hook(CompilerCompilation for HotModuleReplacementPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ImportMetaHotAccept,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ImportMetaHotDecline,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ModuleHotAccept,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ModuleHotDecline,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for HotModuleReplacementPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let Some(CompilationRecords {
    old_chunks,
    all_old_runtime,
    old_all_modules,
    old_runtime_modules,
    old_hash,
  }) = compilation.records.take()
  else {
    return Ok(());
  };

  let mut hot_update_main_content_by_runtime = all_old_runtime
    .iter()
    .map(|runtime| {
      (
        runtime.to_string(),
        HotUpdateContent::new(RuntimeSpec::from_iter([runtime.clone()])),
      )
    })
    .collect::<HashMap<String, HotUpdateContent>>();

  // ----
  if hot_update_main_content_by_runtime.is_empty() {
    return Ok(());
  }

  let (now_all_modules, now_runtime_modules) = collect_changed_modules(compilation)?;

  let mut updated_modules: IdentifierSet = Default::default();
  let mut updated_runtime_modules: IdentifierSet = Default::default();
  let mut completely_removed_modules: HashSet<String> = Default::default();
  let mut updated_chunks: HashMap<ChunkUkey, HashSet<String>> = Default::default();

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

  for (chunk_id, old_runtime) in &old_chunks {
    let mut new_modules = vec![];
    let mut new_runtime_modules = vec![];
    let mut chunk_id = chunk_id.to_string();
    let mut new_runtime = all_old_runtime.clone();
    let mut removed_from_runtime = all_old_runtime.clone();
    let current_chunk = compilation
      .chunk_by_ukey
      .iter()
      .find(|(_, chunk)| chunk.expect_id().eq(&chunk_id))
      .map(|(_, chunk)| chunk);
    let current_chunk_ukey = current_chunk.map(|c| c.ukey);

    if let Some(current_chunk) = current_chunk {
      chunk_id = current_chunk.expect_id().to_string();
      new_runtime = Default::default();
      // intersectRuntime
      for old_runtime in all_old_runtime.iter() {
        if current_chunk.runtime.contains(old_runtime) {
          new_runtime.insert(old_runtime.clone());
        }
      }
      // ------
      if new_runtime.is_empty() {
        continue;
      }

      new_modules = compilation
        .chunk_graph
        .get_chunk_graph_chunk(&current_chunk.ukey)
        .modules
        .iter()
        .filter_map(|module| updated_modules.contains(module).then_some(*module))
        .collect::<Vec<_>>();

      new_runtime_modules = compilation
        .chunk_graph
        .get_chunk_runtime_modules_in_order(&current_chunk.ukey, compilation)
        .filter(|(module, _)| updated_runtime_modules.contains(module))
        .map(|(&module, _)| module)
        .collect::<Vec<_>>();

      // subtractRuntime
      removed_from_runtime = removed_from_runtime.subtract(&new_runtime);
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
      let mut hot_update_chunk = Chunk::new(None, ChunkKind::HotUpdate);
      hot_update_chunk.id = Some(chunk_id.to_string());
      hot_update_chunk.runtime = new_runtime.clone();
      let mut chunk_hash = RspackHash::from(&compilation.options.output);
      let ukey = hot_update_chunk.ukey;
      if let Some(current_chunk) = current_chunk {
        current_chunk
          .groups
          .iter()
          .for_each(|group| hot_update_chunk.add_group(*group))
      }

      for module_identifier in new_modules.iter() {
        if let Some(module) = compilation
          .get_module_graph()
          .module_by_identifier(module_identifier)
        {
          module.hash(&mut chunk_hash);
        }
      }
      let digest = chunk_hash.digest(&compilation.options.output.hash_digest);
      hot_update_chunk
        .content_hash
        .insert(SourceType::JavaScript, digest.clone());
      hot_update_chunk
        .content_hash
        .insert(SourceType::Css, digest);

      compilation.chunk_by_ukey.add(hot_update_chunk);
      compilation.chunk_graph.add_chunk(ukey);

      for module_identifier in new_modules.iter() {
        compilation
          .chunk_graph
          .connect_chunk_and_module(ukey, *module_identifier);
      }

      for runtime_module in new_runtime_modules {
        compilation
          .chunk_graph
          .connect_chunk_and_runtime_module(ukey, runtime_module);
      }

      let mut manifest = Vec::new();
      let mut diagnostics = Vec::new();
      compilation
        .plugin_driver
        .compilation_hooks
        .render_manifest
        .call(compilation, &ukey, &mut manifest, &mut diagnostics)
        .await?;

      compilation.push_batch_diagnostic(diagnostics);

      for entry in manifest {
        let filename = if entry.has_filename() {
          entry.filename().to_string()
        } else {
          let chunk = compilation.chunk_by_ukey.expect_get(&ukey);
          compilation
            .get_path(
              &compilation.options.output.hot_update_chunk_filename,
              PathData::default().chunk(chunk).hash_optional(
                old_hash
                  .as_ref()
                  .map(|hash| hash.rendered(compilation.options.output.hash_digest_length)),
              ),
            )
            .always_ok()
        };
        let asset = CompilationAsset::new(
          Some(entry.source),
          // Reset version to make hmr generated assets always emit
          entry
            .info
            .with_hot_module_replacement(true)
            .with_version(Default::default()),
        );
        if let Some(current_chunk_ukey) = current_chunk_ukey {
          updated_chunks
            .entry(current_chunk_ukey)
            .or_default()
            .insert(filename.clone());
        }
        compilation.emit_asset(filename, asset);
      }

      new_runtime.iter().for_each(|runtime| {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime.as_ref()) {
          info.updated_chunk_ids.insert(chunk_id.to_string());
        }
      });
    }
  }

  // update chunk files
  for (chunk_ukey, files) in updated_chunks {
    compilation
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey)
      .files
      .extend(files);
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
    let filename = compilation
      .get_path(
        &compilation.options.output.hot_update_main_filename,
        PathData::default().runtime(&content.runtime).hash_optional(
          old_hash
            .as_ref()
            .map(|hash| hash.rendered(compilation.options.output.hash_digest_length)),
        ),
      )
      .always_ok();
    compilation.emit_asset(
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

  Ok(())
}

#[plugin_hook(NormalModuleLoader for HotModuleReplacementPlugin)]
fn normal_module_loader(&self, context: &mut LoaderContext<CompilerContext>) -> Result<()> {
  context.hot = true;
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for HotModuleReplacementPlugin)]
fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // TODO: the hmr runtime is depend on module.id, but webpack not add it.
  runtime_requirements.insert(RuntimeGlobals::MODULE_ID);
  runtime_requirements.insert(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
  runtime_requirements.insert(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
  runtime_requirements.insert(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
  runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
  compilation.add_runtime_module(
    chunk_ukey,
    HotModuleReplacementRuntimeModule::default().boxed(),
  )?;

  Ok(())
}

#[async_trait]
impl Plugin for HotModuleReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.HotModuleReplacementPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.dev_server.hot = true;
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    ctx
      .context
      .normal_module_hooks
      .loader
      .tap(normal_module_loader::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}

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
