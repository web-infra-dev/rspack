mod hot_module_replacement;

use std::collections::hash_map;

use atomic_refcell::AtomicRefCell;
use hot_module_replacement::HotModuleReplacementRuntimeModule;
use rspack_collections::IdentifierSet;
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAsset, CompilationId, CompilationParams,
  CompilationProcessAssets, CompilationRecords, CompilerAfterEmit, CompilerCompilation,
  CompilerEmit, DependencyType, LoaderContext, ManifestAssetType, ModuleId, ModuleIdentifier,
  ModuleType, NormalModuleFactoryParser, NormalModuleLoader, ParserAndGenerator, ParserOptions,
  PathData, Plugin, RunnerContext, RuntimeGlobals, RuntimeModule, RuntimeModuleExt, RuntimeSpec,
  chunk_graph_chunk::{ChunkId, ChunkIdMap, ChunkIdSet},
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::{RspackHashDigest, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::parser_and_generator::CssParserAndGenerator;
use rspack_plugin_javascript::{
  hot_module_replacement_plugin::{
    ImportMetaHotReplacementParserPlugin, ModuleHotReplacementParserPlugin,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use ustr::Ustr;

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// Each compiler owns its plugin instance and drives one compilation at a
/// time, and this plugin's hooks (processAssets, emit, afterEmit) run
/// strictly in sequence within it, so the fields are never borrowed
/// concurrently.
#[plugin]
#[derive(Debug)]
pub struct HotModuleReplacementPlugin {
  // js parts of the update are computed early (so hot-update chunks still
  // pass through the later processAssets stages, e.g. for source maps), the
  // manifest waits for the final assets to diff the emitted css; a single
  // slot, so an entry left behind by an aborted build is overwritten by the
  // next one instead of accumulating
  js_hot_update: AtomicRefCell<Option<(CompilationId, JsHotUpdate)>>,
  // per-chunk digests of the css assets of the last fully emitted build: the
  // old side of the diff, matching the newest stylesheets the browser can
  // hold; a build that fails or skips emission must not advance it, or the
  // next successful build would under-report its css changes
  previous_css_hashes: AtomicRefCell<ChunkIdMap<ChunkCssHashes>>,
  // the current build's snapshot, staged at emit and committed into
  // `previous_css_hashes` by after_emit once every asset is written
  staged_css_hashes: AtomicRefCell<Option<(CompilationId, ChunkIdMap<ChunkCssHashes>)>>,
}

impl Default for HotModuleReplacementPlugin {
  fn default() -> Self {
    Self::new_inner(Default::default(), Default::default(), Default::default())
  }
}

#[derive(Debug)]
struct JsHotUpdate {
  content_by_runtime: HashMap<Ustr, HotUpdateContent>,
  css_diff_tasks: Vec<CssDiffTask>,
  completely_removed_modules: HashSet<ModuleId>,
  old_hash: Option<RspackHashDigest>,
}

#[derive(Debug)]
struct CssDiffTask {
  chunk_id: ChunkId,
  in_new_compilation: bool,
  new_runtime: RuntimeSpec,
  removed_from_runtime: RuntimeSpec,
}

/// Digests of a chunk's emitted CSS assets, taken from the final rendered
/// asset content so transforms applied during `processAssets` are covered.
/// `None` when the chunk has no CSS of that kind. They feed the manifest
/// `css` (native css runtime) and `miniCss` (CssExtractRspackPlugin) fields,
/// which are kept apart so an update of one runtime's CSS never makes the
/// other runtime fetch a stylesheet it does not own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ChunkCssHashes {
  css: Option<RspackHashDigest>,
  mini_css: Option<RspackHashDigest>,
}

impl ChunkCssHashes {
  fn is_empty(&self) -> bool {
    self.css.is_none() && self.mini_css.is_none()
  }

  fn from_chunk_assets<'c>(
    compilation: &'c Compilation,
    chunk: &'c Chunk,
    file_digests: &mut HashMap<&'c str, RspackHashDigest>,
  ) -> Self {
    let mut css_files: Vec<&str> = Vec::new();
    let mut mini_css_files: Vec<&str> = Vec::new();
    for file in chunk.files() {
      let Some(asset) = compilation.assets().get(file) else {
        continue;
      };
      // a source-less asset (`delete compilation.assets[f]` only drops the
      // source) is never emitted, so its stylesheet no longer exists
      if asset.get_source().is_none() {
        continue;
      }
      match &asset.info.asset_type {
        ManifestAssetType::Css => css_files.push(file),
        // the asset type CssExtractRspackPlugin tags its stylesheets with
        ManifestAssetType::Custom(name) if name == "extract-css" => mini_css_files.push(file),
        _ => {}
      }
    }
    Self {
      css: digest_asset_contents(compilation, css_files, file_digests),
      mini_css: digest_asset_contents(compilation, mini_css_files, file_digests),
    }
  }
}

fn digest_asset_contents<'c>(
  compilation: &'c Compilation,
  mut files: Vec<&'c str>,
  file_digests: &mut HashMap<&'c str, RspackHashDigest>,
) -> Option<RspackHashDigest> {
  if files.is_empty() {
    return None;
  }
  files.sort_unstable();
  let mut hasher = RspackHasher::from(&compilation.options.output);
  for file in files {
    let digest = file_digests.entry(file).or_insert_with(|| {
      let mut hasher = RspackHasher::from(&compilation.options.output);
      if let Some(source) = compilation.assets().get(file).and_then(|a| a.get_source()) {
        hasher.write(source.buffer().as_ref());
      }
      hasher.digest(&compilation.options.output.hash_digest)
    });
    // fixed-length per-file digests keep the concatenation unambiguous
    hasher.write(digest.encoded().as_bytes());
  }
  Some(hasher.digest(&compilation.options.output.hash_digest))
}

fn snapshot_chunk_css_hashes(compilation: &Compilation) -> ChunkIdMap<ChunkCssHashes> {
  // per-asset digests are memoized so a stylesheet shared by several chunks
  // is hashed once per build
  let mut file_digests: HashMap<&str, RspackHashDigest> = Default::default();
  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.kind() != ChunkKind::HotUpdate)
    .filter_map(|chunk| {
      let css_hashes = ChunkCssHashes::from_chunk_assets(compilation, chunk, &mut file_digests);
      if css_hashes.is_empty() {
        return None;
      }
      Some((chunk.expect_id().clone(), css_hashes))
    })
    .collect()
}

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
  let Some(records) = compilation.records.take() else {
    return Ok(());
  };
  let CompilationRecords {
    chunks: old_chunks,
    runtimes: all_old_runtime,
    modules: old_all_modules,
    runtime_modules: old_runtime_modules,
    hash: old_hash,
  } = records.as_ref();

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
  let mut updated_chunks: HashMap<ChunkUkey, HashSet<String>> = Default::default();
  for (identifier, old_runtime_module_hash) in old_runtime_modules {
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
  let mut css_diff_tasks: Vec<CssDiffTask> = Default::default();

  for (chunk_id, (old_runtime, old_module_ids)) in old_chunks {
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

    let current_chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .iter()
      .find(|(_, chunk)| chunk.expect_id().eq(&chunk_id))
      .map(|(_, chunk)| chunk);
    let current_chunk_ukey = current_chunk.map(|c| c.ukey());

    if let Some(current_chunk) = current_chunk {
      new_runtime = current_chunk
        .runtime()
        .intersection(all_old_runtime)
        .copied()
        .collect();

      if new_runtime.is_empty() {
        continue;
      }

      new_modules = compilation
        .build_chunk_graph_artifact
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

      new_runtime_modules = compilation
        .build_chunk_graph_artifact
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

    // the css side of the diff runs against the final assets in the late
    // manifest hook; collect what it needs while the js diff is at hand
    css_diff_tasks.push(CssDiffTask {
      chunk_id: chunk_id.clone(),
      in_new_compilation: current_chunk.is_some(),
      new_runtime: new_runtime.clone(),
      removed_from_runtime: removed_from_runtime.clone(),
    });

    for old_module_id in remaining_modules {
      let module_identifier = all_module_ids
        .get(&old_module_id)
        .expect("should have module");
      let old_hashes = old_all_modules
        .get(&old_module_id)
        .expect("should have module");
      let old_hash = old_hashes.get(&chunk_id);
      let runtimes = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_runtimes(
          *module_identifier,
          &compilation.build_chunk_graph_artifact.chunk_by_ukey,
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
      compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .add(hot_update_chunk);

      // In webpack, compilation.chunkGraph uses a WeakMap to maintain the relationship between Chunks and Modules.
      // This means the lifecycle of these data is tied to the Chunk, and they are garbage-collected when the Chunk is.
      //
      // In Rspack, we need to manually clean up the data in compilation.build_chunk_graph_artifact.chunk_graph after HotUpdateChunk is used.
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .add_chunk(ukey);
      for module_identifier in &new_modules {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .connect_chunk_and_module(ukey, *module_identifier);
      }
      for runtime_module in &new_runtime_modules {
        compilation.code_generated_modules.insert(*runtime_module);
        compilation
          .build_chunk_graph_artifact
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
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_module(&ukey, module_identifier);
      }
      for runtime_module in new_runtime_modules {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_runtime_module(&ukey, &runtime_module);
      }
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .remove_chunk(&ukey);
      #[allow(clippy::unwrap_used)]
      let hot_update_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .remove(&ukey)
        .unwrap();

      compilation.extend_diagnostics(diagnostics);

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
        compilation.emit_asset(filename, asset);
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
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey);
    for file in files {
      chunk.add_file(file);
    }
  }

  *self.js_hot_update.borrow_mut() = Some((
    compilation.id(),
    JsHotUpdate {
      content_by_runtime: hot_update_main_content_by_runtime,
      css_diff_tasks,
      completely_removed_modules,
      old_hash: old_hash.clone(),
    },
  ));

  Ok(())
}

#[plugin_hook(CompilerEmit for HotModuleReplacementPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  // this hook runs after every asset transformation (all processAssets
  // stages, afterProcessAssets, afterSeal) and right before assets are
  // written, so the snapshot holds exactly the bytes the browser will fetch;
  // it is taken on every build and becomes the next rebuild's old side once
  // after_emit commits it
  let current_css_hashes = snapshot_chunk_css_hashes(compilation);
  *self.staged_css_hashes.borrow_mut() = Some((compilation.id(), current_css_hashes.clone()));

  let Some((js_update_compilation_id, js_update)) = self.js_hot_update.borrow_mut().take() else {
    return Ok(());
  };
  // a stale entry from a build that aborted after the js diff must not be
  // emitted against this compilation's assets
  if js_update_compilation_id != compilation.id() {
    return Ok(());
  }
  let JsHotUpdate {
    mut content_by_runtime,
    css_diff_tasks,
    completely_removed_modules,
    old_hash,
  } = js_update;

  let old_chunk_css_hashes = self.previous_css_hashes.borrow();
  for task in css_diff_tasks {
    let old_css_hashes = old_chunk_css_hashes.get(&task.chunk_id);
    let new_css_hashes = if task.in_new_compilation {
      current_css_hashes.get(&task.chunk_id)
    } else {
      None
    };
    let css_update = CssUpdate::new(
      old_css_hashes.and_then(|hashes| hashes.css.as_ref()),
      new_css_hashes.and_then(|hashes| hashes.css.as_ref()),
    );
    let mini_css_update = CssUpdate::new(
      old_css_hashes.and_then(|hashes| hashes.mini_css.as_ref()),
      new_css_hashes.and_then(|hashes| hashes.mini_css.as_ref()),
    );

    for removed in task.removed_from_runtime.iter() {
      if let Some(info) = content_by_runtime.get_mut(removed) {
        if old_css_hashes.is_some_and(|hashes| hashes.css.is_some()) {
          info.css_removed_chunk_ids.insert(task.chunk_id.clone());
        }
        if old_css_hashes.is_some_and(|hashes| hashes.mini_css.is_some()) {
          info
            .mini_css_removed_chunk_ids
            .insert(task.chunk_id.clone());
        }
      }
    }

    // Independent of whether the chunk carries updated js modules: a chunk
    // holding only extracted css has no js update when its stylesheet changes.
    if task.in_new_compilation {
      for runtime in task.new_runtime.iter() {
        if let Some(info) = content_by_runtime.get_mut(runtime) {
          if css_update == CssUpdate::Removed {
            info.css_removed_chunk_ids.insert(task.chunk_id.clone());
          }
          if mini_css_update == CssUpdate::Removed {
            info
              .mini_css_removed_chunk_ids
              .insert(task.chunk_id.clone());
          }
          if css_update == CssUpdate::Changed {
            info.css_updated_chunk_ids.insert(task.chunk_id.clone());
          }
          if mini_css_update == CssUpdate::Changed {
            info
              .mini_css_updated_chunk_ids
              .insert(task.chunk_id.clone());
          }
        }
      }
    }
  }
  // released before the awaits below: an AtomicRefCell guard must not be
  // held across suspension points
  drop(old_chunk_css_hashes);

  let mut hot_update_main_content_by_filename = HashMap::default();
  for (runtime, content) in content_by_runtime {
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
        old_content
          .css_updated_chunk_ids
          .extend(content.css_updated_chunk_ids);
        old_content
          .css_removed_chunk_ids
          .extend(content.css_removed_chunk_ids);
        old_content
          .mini_css_updated_chunk_ids
          .extend(content.mini_css_updated_chunk_ids);
        old_content
          .mini_css_removed_chunk_ids
          .extend(content.mini_css_removed_chunk_ids);
        compilation.push_diagnostic(Diagnostic::warn(
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

    let mut manifest_json = serde_json::json!({
      "c": c,
      "r": r,
      "m": m,
    });
    if let Some(css) =
      css_manifest_json(content.css_updated_chunk_ids, content.css_removed_chunk_ids)
    {
      manifest_json["css"] = css;
    }
    if let Some(mini_css) = css_manifest_json(
      content.mini_css_updated_chunk_ids,
      content.mini_css_removed_chunk_ids,
    ) {
      manifest_json["miniCss"] = mini_css;
    }
    let manifest_content = manifest_json.to_string();

    compilation.emit_asset(
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
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule
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

#[plugin_hook(CompilerAfterEmit for HotModuleReplacementPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  // commit the snapshot only once every asset is written: a build that
  // failed or skipped emission must not become the old side of the next
  // diff, the browser never saw its stylesheets
  let staged = self.staged_css_hashes.borrow_mut().take();
  if let Some((staged_compilation_id, snapshot)) = staged
    && staged_compilation_id == compilation.id()
  {
    *self.previous_css_hashes.borrow_mut() = snapshot;
  }
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
    ctx.compiler_hooks.emit.tap(emit::new(self));
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));
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

#[derive(Debug, Default)]
struct HotUpdateContent {
  updated_chunk_ids: ChunkIdSet,
  removed_chunk_ids: ChunkIdSet,
  removed_modules: HashSet<ModuleId>,
  css_updated_chunk_ids: ChunkIdSet,
  css_removed_chunk_ids: ChunkIdSet,
  mini_css_updated_chunk_ids: ChunkIdSet,
  mini_css_removed_chunk_ids: ChunkIdSet,
}

#[derive(Clone, Copy, PartialEq)]
enum CssUpdate {
  Changed,
  Removed,
  Unchanged,
}

impl CssUpdate {
  fn new(old: Option<&RspackHashDigest>, new: Option<&RspackHashDigest>) -> Self {
    match (old, new) {
      (old, Some(new)) if old != Some(new) => Self::Changed,
      (Some(_), None) => Self::Removed,
      _ => Self::Unchanged,
    }
  }
}

fn css_manifest_json(updated: ChunkIdSet, removed: ChunkIdSet) -> Option<serde_json::Value> {
  if updated.is_empty() && removed.is_empty() {
    return None;
  }
  let mut css = serde_json::Map::new();
  if !updated.is_empty() {
    let c: Vec<ChunkId> = updated.into_iter().collect();
    css.insert("c".to_string(), serde_json::json!(c));
  }
  if !removed.is_empty() {
    let r: Vec<ChunkId> = removed.into_iter().collect();
    css.insert("r".to_string(), serde_json::json!(r));
  }
  Some(serde_json::Value::Object(css))
}
