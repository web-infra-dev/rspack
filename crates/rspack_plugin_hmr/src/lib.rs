mod hot_module_replacement;

use std::collections::hash_map;

use hot_module_replacement::HotModuleReplacementRuntimeModule;
use rspack_collections::{DatabaseItem, IdentifierSet, UkeyMap};
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAsset, CompilationParams,
  CompilationProcessAssets, CompilationRecords, CompilerCompilation, DependencyType, LoaderContext,
  ModuleId, ModuleIdentifier, ModuleType, NormalModuleFactoryParser, NormalModuleLoader,
  ParserAndGenerator, ParserOptions, PathData, Plugin, RunnerContext, RuntimeGlobals,
  RuntimeModule, RuntimeModuleExt, RuntimeSpec,
  chunk_graph_chunk::ChunkId,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::parser_and_generator::CssParserAndGenerator;
use rspack_plugin_javascript::{
  hot_module_replacement_plugin::{
    ImportMetaHotReplacementParserPlugin, ModuleHotReplacementParserPlugin,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};
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
async fn process_assets(&self, compilation: &Compilation, artifact: &mut rspack_core::ProcessAssetsArtifact, build_chunk_graph_artifact: &mut rspack_core::BuildChunkGraphArtifact) -> Result<()> {
  let Some(CompilationRecords {
    chunks: old_chunks,
    runtimes: all_old_runtime,
    modules: old_all_modules,
    runtime_modules: old_runtime_modules,
    hash: old_hash,
  }) = artifact.records.take()
  else {
    return Ok(());
  };

  if let Some(old_hash) = &old_hash
    && let Some(hash) = &compilation.hash
    && old_hash == hash
  {
    return Ok(());
  }

  let mut hot_update_main_content_by_runtime = all_old_runtime
    .iter()
    .map(|&runtime| (runtime, HotUpdateContent::default()))
    .collect::<HashMap<_, HotUpdateContent>>();

  if hot_update_main_content_by_runtime.is_empty() {
    return Ok(());
  }

  let mut updated_runtime_modules: IdentifierSet = Default::default();
  let mut updated_chunks: UkeyMap<ChunkUkey, HashSet<String>> = Default::default();
  for (identifier, old_runtime_module_hash) in &old_runtime_modules {
    if let Some(new_runtime_module_hash) = compilation.runtime_modules_hash.get(identifier) {
      // updated
      if new_runtime_module_hash != old_runtime_module_hash {
        updated_runtime_modules.insert(*identifier);
      }
    }
  }
  for identifier in compilation.runtime_modules.keys() {
    if !old_runtime_modules.contains_key(identifier) {
      // added
      updated_runtime_modules.insert(*identifier);
    }
  }

  let all_module_ids: HashMap<ModuleId, ModuleIdentifier> = compilation
    .module_ids_artifact
    .iter()
    .map(|(k, v)| (v.clone(), *k))
    .collect();
  let mut completely_removed_modules: HashSet<ModuleId> = Default::default();

  for (chunk_id, (old_runtime, old_module_ids)) in &old_chunks {
    let mut remaining_modules: HashSet<ModuleId> = Default::default();
    for old_module_id in old_module_ids {
      if !all_module_ids.contains_key(old_module_id) {
        completely_removed_modules.insert(old_module_id.clone());
      } else {
        remaining_modules.insert(old_module_id.clone());
      }
    }

    let mut new_modules = vec![];
    let mut new_runtime_modules = vec![];
    let chunk_id = chunk_id.clone();
    let new_runtime: RuntimeSpec;
    let removed_from_runtime: RuntimeSpec;

    let current_chunk = build_chunk_graph_artifact
      .chunk_by_ukey
      .iter()
      .find(|(_, chunk)| chunk.expect_id().eq(&chunk_id))
      .map(|(_, chunk)| chunk);
    let current_chunk_ukey = current_chunk.map(|c| c.ukey());

    if let Some(current_chunk) = current_chunk {
      new_runtime = current_chunk
        .runtime()
        .intersection(&all_old_runtime)
        .copied()
        .collect();

      if new_runtime.is_empty() {
        continue;
      }

      new_modules = build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(&current_chunk.ukey())
        .iter()
        .filter_map(|&module| {
          let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)?;
          let Some(old_module_hashes) = old_all_modules.get(module_id) else {
            return Some(module);
          };
          let old_hash = old_module_hashes.get(&chunk_id);
          let new_hash = compilation
            .code_generation_results
            .get_hash(&module, Some(current_chunk.runtime()));
          if old_hash != new_hash {
            return Some(module);
          }
          None
        })
        .collect::<Vec<_>>();

      new_runtime_modules = build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_in_order(&current_chunk.ukey(), compilation)
        .filter(|(module, _)| updated_runtime_modules.contains(module))
        .map(|(&module, _)| module)
        .collect::<Vec<_>>();

      removed_from_runtime = old_runtime.subtract(&new_runtime);
    } else {
      removed_from_runtime = old_runtime.clone();
      new_runtime = old_runtime.clone();
    }

    for removed in removed_from_runtime.iter() {
      if let Some(info) = hot_update_main_content_by_runtime.get_mut(removed) {
        info.removed_chunk_ids.insert(chunk_id.clone());
      }
    }

    for old_module_id in remaining_modules {
      let module_identifier = all_module_ids
        .get(&old_module_id)
        .expect("should have module");
      let old_hashes = old_all_modules
        .get(&old_module_id)
        .expect("should have module");
      let old_hash = old_hashes.get(&chunk_id);
      let runtimes = build_chunk_graph_artifact
        .chunk_graph
        .get_module_runtimes(
          *module_identifier,
          &build_chunk_graph_artifact.chunk_by_ukey,
        );
      if old_runtime == &new_runtime && runtimes.contains(&new_runtime) {
        let new_hash = compilation
          .code_generation_results
          .get_hash(module_identifier, Some(&new_runtime));
        if new_hash != old_hash {
          new_modules.push(*module_identifier);
        }
      } else {
        for removed in removed_from_runtime.iter() {
          if let Some(content) = hot_update_main_content_by_runtime.get_mut(removed) {
            content.removed_modules.insert(old_module_id.clone());
          }
        }
      }
    }

    if !new_modules.is_empty() || !new_runtime_modules.is_empty() {
      let mut hot_update_chunk = Chunk::new(None, ChunkKind::HotUpdate);
      hot_update_chunk.set_id(chunk_id.clone());
      hot_update_chunk.set_runtime(if let Some(current_chunk) = current_chunk {
        current_chunk.runtime().clone()
      } else {
        new_runtime.clone()
      });
      let ukey = hot_update_chunk.ukey();

      if let Some(current_chunk) = current_chunk {
        current_chunk
          .groups()
          .iter()
          .for_each(|group| hot_update_chunk.add_group(*group))
      }

      // In webpack, there is no need to add HotUpdateChunk to compilation.chunks,
      // because HotUpdateChunk is no longer used after generating the manifest.
      //
      // However, in Rspack, we need to add HotUpdateChunk to compilation.build_chunk_graph_artifact.chunk_by_ukey
      // because during the manifest generation, HotUpdateChunk is passed to various plugins via the ukey.
      // The plugins then use the ukey to query compilation.build_chunk_graph_artifact.chunk_by_ukey to get the HotUpdateChunk instance.
      // Therefore, in Rspack, after the manifest is generated, we need to manually remove the HotUpdateChunk from compilation.chunks.
      build_chunk_graph_artifact
        .chunk_by_ukey
        .add(hot_update_chunk);

      // In webpack, compilation.chunkGraph uses a WeakMap to maintain the relationship between Chunks and Modules.
      // This means the lifecycle of these data is tied to the Chunk, and they are garbage-collected when the Chunk is.
      //
      // In Rspack, we need to manually clean up the data in compilation.build_chunk_graph_artifact.chunk_graph after HotUpdateChunk is used.
      build_chunk_graph_artifact
        .chunk_graph
        .add_chunk(ukey);
      for module_identifier in &new_modules {
        build_chunk_graph_artifact
          .chunk_graph
          .connect_chunk_and_module(ukey, *module_identifier);
      }
      for runtime_module in &new_runtime_modules {
        artifact.code_generated_modules.insert(*runtime_module);
        build_chunk_graph_artifact
          .chunk_graph
          .connect_chunk_and_runtime_module(ukey, *runtime_module);
      }

      let mut manifest = Vec::new();
      let mut diagnostics = Vec::new();
      compilation
        .plugin_driver
        .compilation_hooks
        .render_manifest
        .call(compilation, &ukey, &mut manifest, &mut diagnostics)
        .await?;

      // Manually clean up ChunkGraph and chunks
      for module_identifier in new_modules {
        build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_module(&ukey, module_identifier);
      }
      for runtime_module in new_runtime_modules {
        build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_runtime_module(&ukey, &runtime_module);
      }
      build_chunk_graph_artifact
        .chunk_graph
        .remove_chunk(&ukey);
      #[allow(clippy::unwrap_used)]
      let hot_update_chunk = build_chunk_graph_artifact
        .chunk_by_ukey
        .remove(&ukey)
        .unwrap();

      artifact.diagnostics.extend(diagnostics);

      for entry in manifest {
        let filename = if entry.has_filename {
          entry.filename.clone()
        } else {
          compilation
            .get_path(
              &compilation.options.output.hot_update_chunk_filename,
              PathData::default()
                .chunk_id_optional(hot_update_chunk.id().map(|id| id.as_str()))
                .chunk_name_optional(hot_update_chunk.name_for_filename_template())
                .hash_optional(
                  old_hash
                    .as_ref()
                    .map(|hash| hash.rendered(compilation.options.output.hash_digest_length)),
                ),
            )
            .await?
        };
        let asset = CompilationAsset::new(
          Some(entry.source),
          // Reset version to make hmr generated assets always emit
          entry
            .info
            .with_hot_module_replacement(Some(true))
            .with_version(Default::default()),
        );
        if let Some(current_chunk_ukey) = current_chunk_ukey {
          updated_chunks
            .entry(current_chunk_ukey)
            .or_default()
            .insert(filename.clone());
        }
        artifact.assets.insert(filename, asset);
      }

      new_runtime.iter().for_each(|runtime| {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
          info.updated_chunk_ids.insert(chunk_id.clone());
        }
      });
    }
  }

  // update chunk files
  for (chunk_ukey, files) in updated_chunks {
    let chunk = build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey);
    for file in files {
      chunk.add_file(file);
    }
  }

  let mut hot_update_main_content_by_filename = HashMap::default();
  for (runtime, content) in hot_update_main_content_by_runtime {
    let filename = compilation
      .get_path(
        &compilation.options.output.hot_update_main_filename,
        PathData::default().runtime(&runtime).hash_optional(
          old_hash
            .as_ref()
            .map(|hash| hash.rendered(compilation.options.output.hash_digest_length)),
        ),
      )
      .await?;
    match hot_update_main_content_by_filename.entry(filename) {
      hash_map::Entry::Occupied(mut occupied_entry) => {
        let old_content: &mut HotUpdateContent = occupied_entry.get_mut();
        old_content
          .updated_chunk_ids
          .extend(content.updated_chunk_ids);
        old_content
          .removed_chunk_ids
          .extend(content.removed_chunk_ids);
        old_content.removed_modules.extend(content.removed_modules);
        artifact.diagnostics.push(Diagnostic::warn(
          "HotModuleReplacementPlugin".to_string(),
          r#"The configured output.hotUpdateMainFilename doesn't lead to unique filenames per runtime and HMR update differs between runtimes.
This might lead to incorrect runtime behavior of the applied update.
To fix this, make sure to include [runtime] in the output.hotUpdateMainFilename option, or use the default config."#.to_string(),
        ));
      }
      hash_map::Entry::Vacant(vacant_entry) => {
        vacant_entry.insert(content);
      }
    }
  }
  for (filename, content) in hot_update_main_content_by_filename {
    let c: Vec<ChunkId> = content.updated_chunk_ids.into_iter().collect();
    let r: Vec<ChunkId> = content.removed_chunk_ids.into_iter().collect();
    let m: Vec<ModuleId> = {
      let mut m = completely_removed_modules.clone();
      m.extend(content.removed_modules);
      m.into_iter().collect()
    };

    let manifest_content = serde_json::json!({
      "c": c,
      "r": r,
      "m": m,
    })
    .to_string();

    artifact.assets.insert(
      filename,
      CompilationAsset::new(
        Some(
          RawStringSource::from(if compilation.options.output.module {
            format!("export default {manifest_content};")
          } else {
            manifest_content
          })
          .boxed(),
        ),
        AssetInfo::default().with_hot_module_replacement(Some(true)),
      ),
    );
  }

  Ok(())
}

#[plugin_hook(NormalModuleLoader for HotModuleReplacementPlugin)]
async fn normal_module_loader(&self, context: &mut LoaderContext<RunnerContext>) -> Result<()> {
  context.hot = true;
  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for HotModuleReplacementPlugin)]
async fn normal_module_factory_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>() {
    if module_type.is_js_auto() {
      parser.add_parser_plugin(Box::new(ModuleHotReplacementParserPlugin::new()));
      parser.add_parser_plugin(Box::new(ImportMetaHotReplacementParserPlugin::new()));
    } else if module_type.is_js_dynamic() {
      parser.add_parser_plugin(Box::new(ModuleHotReplacementParserPlugin::new()));
    } else if module_type.is_js_esm() {
      parser.add_parser_plugin(Box::new(ImportMetaHotReplacementParserPlugin::new()));
    }
  } else if matches!(
    module_type,
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssModule
  ) && let Some(parser) = parser.downcast_mut::<CssParserAndGenerator>()
  {
    parser.hot = true;
  }

  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for HotModuleReplacementPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  runtime_modules
    .push(HotModuleReplacementRuntimeModule::new(&compilation.runtime_template).boxed());

  Ok(())
}

impl Plugin for HotModuleReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.HotModuleReplacementPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    ctx
      .normal_module_hooks
      .loader
      .tap(normal_module_loader::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(normal_module_factory_parser::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}

#[derive(Default)]
struct HotUpdateContent {
  updated_chunk_ids: HashSet<ChunkId>,
  removed_chunk_ids: HashSet<ChunkId>,
  removed_modules: HashSet<ModuleId>,
}
