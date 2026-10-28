mod hot_module_replacement;

use std::{
  collections::{BTreeMap, hash_map},
  sync::{Arc, LazyLock, Mutex},
};

use atomic_refcell::AtomicRefCell;
use hot_module_replacement::HotModuleReplacementRuntimeModule;
use rspack_collections::IdentifierSet;
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkInitFragments, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAsset, CompilationContentHash,
  CompilationParams, CompilationProcessAssets, CompilationRecords, CompilerCompilation,
  DependencyType, LoaderContext, Module, ModuleId, ModuleIdentifier, ModuleType,
  NormalModuleFactoryParser, NormalModuleLoader, ParserAndGenerator, ParserOptions, PathData,
  Plugin, RunnerContext, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule, RuntimeModuleExt,
  RuntimeSpec, SourceType,
  chunk_graph_chunk::{ChunkId, ChunkIdMap, ChunkIdSet},
  chunk_graph_module::ModuleIdSet,
  incremental::{IncrementalPasses, Mutation},
  rspack_sources::{RawStringSource, ReplaceSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::{RspackHashDigest, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::parser_and_generator::CssParserAndGenerator;
use rspack_plugin_javascript::{
  JavascriptModulesRenderModuleContainer, JsPlugin, RenderSource,
  hot_module_replacement_plugin::{
    ImportMetaHotReplacementParserPlugin, ModuleHotReplacementParserPlugin,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};
use rspack_plugin_runtime::BlockChunkMapData;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

type BlockChunks = BTreeMap<String, Vec<ChunkId>>;

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// `previous_css_hashes` is only touched from the processAssets hook, which
/// runs serially (each compiler owns its plugin instance and drives one
/// compilation at a time). The content_hash tap runs concurrently across
/// chunks, so `collected_css_hashes` uses a `Mutex` instead.
#[plugin]
#[derive(Debug)]
pub struct HotModuleReplacementPlugin {
  // per-chunk css digests captured by the content_hash tap: the unsalted
  // css-related entries the css plugins feed into the chunk content hash,
  // i.e. derived from exactly the ordered module lists the css assets are
  // rendered from; chunks-hashes is incremental, so entries only arrive for
  // the chunks that were re-hashed this build
  collected_css_hashes: Mutex<ChunkIdMap<ChunkCssHashes>>,
  // the previous sealed build's per-chunk css digests: the old side of the
  // diff, advanced on every build like `CompilationRecords`
  previous_css_hashes: AtomicRefCell<ChunkIdMap<ChunkCssHashes>>,
}

impl Default for HotModuleReplacementPlugin {
  fn default() -> Self {
    Self::new_inner(Default::default(), Default::default())
  }
}

/// Digests of a chunk's CSS content, split per consumer: `css` for the native
/// css runtime, `mini_css` for CssExtractRspackPlugin, kept apart because
/// each feeds its own HMR runtime. `None` means the chunk has no css of that
/// kind.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ChunkCssHashes {
  css: Option<RspackHashDigest>,
  mini_css: Option<RspackHashDigest>,
}

impl ChunkCssHashes {
  fn is_empty(&self) -> bool {
    self.css.is_none() && self.mini_css.is_none()
  }
}

fn is_module_unchanged(compilation: &Compilation, chunk: &Chunk, module: ModuleIdentifier) -> bool {
  let Some(records) = &compilation.records else {
    return false;
  };
  let Some(module_id) = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module) else {
    return false;
  };
  let Some(old_hashes) = records.modules.get(module_id) else {
    return false;
  };
  let new_hash = compilation
    .code_generation_results
    .get_hash(&module, Some(chunk.runtime()));
  // Prefer the same runtime and chunk: other runtime variants may have different factories.
  if let Some(old_hash) = old_hashes.get(chunk.expect_id()).filter(|_| {
    records
      .chunks
      .get(chunk.expect_id())
      .is_some_and(|(runtime, _)| runtime == chunk.runtime())
  }) {
    return Some(old_hash) == new_hash;
  }
  let mut previous = old_hashes
    .iter()
    .filter(|(id, _)| {
      records.chunks.get(*id).is_some_and(|(runtimes, _)| {
        runtimes
          .iter()
          .any(|runtime| records.runtimes.contains(runtime) && chunk.runtime().contains(runtime))
      })
    })
    .peekable();
  previous.peek().is_some() && previous.all(|(_, hash)| Some(hash) == new_hash)
}

#[plugin_hook(CompilerCompilation for HotModuleReplacementPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.hot = true;
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
  JsPlugin::get_compilation_hooks_mut(compilation.id())
    .write()
    .await
    .render_module_container
    .tap(render_module_container::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContainer for HotModuleReplacementPlugin, stage = -100, tracing = false)]
async fn render_module_container(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &dyn Module,
  source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
  _runtime_template: &RuntimeCodeTemplate,
) -> Result<()> {
  let graph = &compilation.build_chunk_graph_artifact;
  let chunk = graph.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.kind() != ChunkKind::HotUpdate {
    return Ok(());
  }
  // A factory forwarded through its old chunk is still an ordinary update.
  let Some(current) = graph
    .chunk_graph
    .get_module_chunks(module.identifier())
    .iter()
    .map(|key| graph.chunk_by_ukey.expect_get(key))
    .find(|current| current.kind() != ChunkKind::HotUpdate && current.id() == chunk.id())
  else {
    return Ok(());
  };
  if !is_module_unchanged(compilation, current, module.identifier()) {
    return Ok(());
  }

  // [factory] installs code without invalidating an unchanged module instance.
  let end = (source.source.size() - 2) as u32;
  let mut replacement = ReplaceSource::new(source.source.clone());
  if compilation
    .options
    .output
    .environment
    .supports_method_shorthand()
  {
    replacement.insert_static(0, ": [(function ", None);
    replacement.insert_static(end, ")]", None);
  } else {
    replacement.insert_static(2, "[", None);
    replacement.insert_static(end, "]", None);
  }
  source.source = replacement.boxed();
  Ok(())
}

// Runs after the css plugins' taps (their default stage is 0) filled the
// css-related entries, and before the chunk-hash salt is applied to them, so
// the digests change exactly when the chunk's css content does.
#[plugin_hook(CompilationContentHash for HotModuleReplacementPlugin, stage = 100)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut HashMap<SourceType, RspackHasher>,
) -> Result<()> {
  // The content-hash key CssExtractRspackPlugin emits under.
  static MINI_EXTRACT_CSS: LazyLock<SourceType> =
    LazyLock::new(|| SourceType::Custom("css/mini-extract".into()));
  let Some(chunk) = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
  else {
    return Ok(());
  };
  if chunk.kind() == ChunkKind::HotUpdate {
    return Ok(());
  }
  let Some(chunk_id) = chunk.id() else {
    return Ok(());
  };
  let output = &compilation.options.output;
  let digest_of = |source_type: &SourceType| {
    hashes
      .get(source_type)
      .map(|hasher| hasher.clone().digest(&output.hash_digest))
  };
  // the native css plugin writes a `SourceType::Css` entry for every chunk,
  // so an empty digest means "no css"; the extract entry only exists when
  // the chunk has extracted css
  let empty_css_digest = RspackHasher::from(output).digest(&output.hash_digest);
  let css_hashes = ChunkCssHashes {
    css: digest_of(&SourceType::Css).filter(|digest| *digest != empty_css_digest),
    mini_css: digest_of(&MINI_EXTRACT_CSS),
  };
  // empty entries are kept: with incremental chunks-hashes only re-hashed
  // chunks arrive here, and a chunk that just lost its css must override its
  // previous entry when the maps are merged
  self
    .collected_css_hashes
    .lock()
    .expect("should lock collected css hashes")
    .insert(chunk_id.clone(), css_hashes);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for HotModuleReplacementPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Advance the css baseline on every sealed build, mirroring the lifecycle
  // of `CompilationRecords`: merge this build's collected digests over the
  // previous snapshot (only re-hashed chunks arrive) and prune chunks gone
  // from this compilation.
  let collected = std::mem::take(
    &mut *self
      .collected_css_hashes
      .lock()
      .expect("should lock collected css hashes"),
  );
  let mut current_css_hashes = self.previous_css_hashes.borrow().clone();
  current_css_hashes.extend(collected);
  let live_chunk_ids: ChunkIdSet = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.kind() != ChunkKind::HotUpdate)
    .filter_map(|chunk| chunk.id().cloned())
    .collect();
  current_css_hashes
    .retain(|chunk_id, css_hashes| !css_hashes.is_empty() && live_chunk_ids.contains(chunk_id));
  let old_chunk_css_hashes = std::mem::replace(
    &mut *self.previous_css_hashes.borrow_mut(),
    current_css_hashes.clone(),
  );

  let Some(records) = compilation.records.as_ref().map(Arc::clone) else {
    return Ok(());
  };
  let CompilationRecords {
    chunks: old_chunks,
    runtimes: all_old_runtime,
    modules: old_all_modules,
    runtime_modules: old_runtime_modules,
    entries: old_entries,
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
      if new_runtime_module_hash != &old_runtime_module_hash.hash {
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
  let current_chunk_ukeys: ChunkIdMap<ChunkUkey> = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .iter()
    .map(|(ukey, chunk)| (chunk.expect_id().clone(), *ukey))
    .collect();
  let completely_removed_modules: HashSet<ModuleId> = old_all_modules
    .iter()
    .filter(|(module_id, chunks)| !chunks.is_empty() && !all_module_ids.contains_key(*module_id))
    .map(|(module_id, _)| module_id.clone())
    .collect();
  let changed_chunks = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNK_ASSET)
    .map(|mutations| {
      mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ChunkSetHashes { chunk } => Some(*chunk),
          _ => None,
        })
        .collect::<HashSet<_>>()
    });

  // Loading changes are independent of factory changes. An unchanged chunk can
  // become a new dependency of an already loaded entry or block.
  let mut full_update_chunks = ChunkIdSet::default();
  for (key, (entry_id, chunks)) in CompilationRecords::record_entries(compilation) {
    let Some((old_entry_id, previous)) = old_entries.get(&key) else {
      continue;
    };
    let needed = chunks
      .into_iter()
      .filter(|id| !previous.contains(id))
      .collect::<Vec<_>>();
    if needed.is_empty() {
      continue;
    }
    let entry = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&current_chunk_ukeys[&entry_id]);
    for runtime in entry.runtime().iter() {
      if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
        info
          .load_entries
          .insert(old_entry_id.clone(), needed.clone());
        full_update_chunks.extend(needed.iter().cloned());
      }
    }
  }
  for (identifier, record) in old_runtime_modules {
    let Some(previous) = record
      .update_data
      .as_deref()
      .and_then(|data| data.as_any().downcast_ref::<BlockChunkMapData>())
    else {
      continue;
    };
    let Some(module) = compilation.runtime_modules.get(identifier) else {
      continue;
    };
    let Some(data) = module.get_update_data(compilation) else {
      continue;
    };
    let Some(current) = data.as_ref().as_any().downcast_ref::<BlockChunkMapData>() else {
      continue;
    };
    let graph = &compilation.build_chunk_graph_artifact;
    let runtime = graph
      .chunk_graph
      .get_runtime_module_chunks(*identifier)
      .iter()
      .flat_map(|key| graph.chunk_by_ukey.expect_get(key).runtime().iter())
      .copied()
      .collect::<RuntimeSpec>();
    for (block_id, chunks) in &current.0 {
      let Some(previous) = previous.0.get(block_id) else {
        continue;
      };
      let needed = chunks
        .iter()
        .filter(|id| !previous.contains(id))
        .cloned()
        .collect::<Vec<_>>();
      if needed.is_empty() {
        continue;
      }
      for runtime in runtime.iter() {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
          info.load_blocks.insert(block_id.clone(), needed.clone());
          full_update_chunks.extend(needed.iter().cloned());
        }
      }
    }
  }

  let empty_runtime = RuntimeSpec::default();
  let empty_modules = ModuleIdSet::default();
  for chunk_id in old_chunks.keys().chain(
    full_update_chunks
      .iter()
      .filter(|id| !old_chunks.contains_key(*id)),
  ) {
    let (old_runtime, old_module_ids) = old_chunks
      .get(chunk_id)
      .map_or((&empty_runtime, &empty_modules), |(runtime, modules)| {
        (runtime, modules)
      });
    let mut new_modules = vec![];
    let mut new_runtime_modules = vec![];
    let chunk_id = chunk_id.clone();
    let new_runtime: RuntimeSpec;
    let removed_from_runtime: RuntimeSpec;

    let current_chunk_ukey = current_chunk_ukeys.get(&chunk_id).copied();
    let current_chunk = current_chunk_ukey.and_then(|ukey| {
      compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get(&ukey)
    });

    if let Some(current_chunk) = current_chunk {
      new_runtime = current_chunk
        .runtime()
        .intersection(all_old_runtime)
        .copied()
        .collect();

      if new_runtime.is_empty() {
        continue;
      }

      if old_runtime == &new_runtime
        && !full_update_chunks.contains(&chunk_id)
        && changed_chunks
          .as_ref()
          .is_some_and(|chunks| !chunks.contains(&current_chunk.ukey()))
      {
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

    let old_css_hashes = old_chunk_css_hashes.get(&chunk_id);
    let new_css_hashes = current_chunk.and_then(|_| current_css_hashes.get(&chunk_id));
    let css_update = CssUpdate::new(
      old_css_hashes.and_then(|hashes| hashes.css.as_ref()),
      new_css_hashes.and_then(|hashes| hashes.css.as_ref()),
    );
    let mini_css_update = CssUpdate::new(
      old_css_hashes.and_then(|hashes| hashes.mini_css.as_ref()),
      new_css_hashes.and_then(|hashes| hashes.mini_css.as_ref()),
    );

    for removed in removed_from_runtime.iter() {
      if let Some(info) = hot_update_main_content_by_runtime.get_mut(removed) {
        info.removed_chunk_ids.insert(chunk_id.clone());
        if old_css_hashes.is_some_and(|hashes| hashes.css.is_some()) {
          info.css_removed_chunk_ids.insert(chunk_id.clone());
        }
        if old_css_hashes.is_some_and(|hashes| hashes.mini_css.is_some()) {
          info.mini_css_removed_chunk_ids.insert(chunk_id.clone());
        }
      }
    }

    // Independent of whether the chunk carries updated js modules: a chunk
    // holding only extracted css has no js update when its stylesheet changes.
    if current_chunk.is_some() {
      for runtime in new_runtime.iter() {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
          if css_update == CssUpdate::Removed {
            info.css_removed_chunk_ids.insert(chunk_id.clone());
          }
          if mini_css_update == CssUpdate::Removed {
            info.mini_css_removed_chunk_ids.insert(chunk_id.clone());
          }
          if css_update == CssUpdate::Changed {
            info.css_updated_chunk_ids.insert(chunk_id.clone());
          }
          if mini_css_update == CssUpdate::Changed {
            info.mini_css_updated_chunk_ids.insert(chunk_id.clone());
          }
        }
      }
    }

    for old_module_id in old_module_ids {
      let Some(module_identifier) = all_module_ids.get(old_module_id) else {
        continue;
      };
      if removed_from_runtime.is_empty()
        && current_chunk_ukey.is_some_and(|ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .is_module_in_chunk(module_identifier, ukey)
        })
      {
        continue;
      }

      let old_hashes = old_all_modules
        .get(old_module_id)
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
          if runtimes.values().any(|runtime| runtime.contains(removed)) {
            continue;
          }
          if let Some(content) = hot_update_main_content_by_runtime.get_mut(removed) {
            content.removed_modules.insert(old_module_id.clone());
          }
        }
      }
    }

    let mut updates = Vec::new();
    if full_update_chunks.contains(&chunk_id)
      && let Some(chunk) = current_chunk
    {
      let graph = &compilation.build_chunk_graph_artifact.chunk_graph;
      let mut modules = graph.get_chunk_modules_identifier(&chunk.ukey()).clone();
      // Keep changed factories forwarded through their previous chunk, too.
      modules.extend(new_modules.iter().copied());
      let runtime_modules = graph
        .get_chunk_runtime_modules_in_order(&chunk.ukey(), compilation)
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
      updates.push((
        true,
        modules.into_iter().collect::<Vec<_>>(),
        runtime_modules,
      ));
    }
    if old_chunks.contains_key(&chunk_id)
      && (!new_modules.is_empty() || !new_runtime_modules.is_empty())
    {
      updates.push((false, new_modules, new_runtime_modules));
    }
    for (full, new_modules, new_runtime_modules) in updates {
      let current_chunk = current_chunk_ukey.and_then(|key| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(&key)
      });
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
        let filename = if full {
          let extension = if filename.ends_with(".mjs") {
            "mjs"
          } else {
            "js"
          };
          format!("{filename}.full.{extension}")
        } else {
          filename
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

      if !full {
        new_runtime.iter().for_each(|runtime| {
          if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
            info.updated_chunk_ids.insert(chunk_id.clone());
          }
        });
      }
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
        old_content.load_entries.extend(content.load_entries);
        old_content.load_blocks.extend(content.load_blocks);
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
    if !content.load_entries.is_empty() || !content.load_blocks.is_empty() {
      let mut load = serde_json::Map::new();
      if !content.load_entries.is_empty() {
        load.insert(
          "entries".into(),
          serde_json::to_value(content.load_entries).expect("serializable chunk IDs"),
        );
      }
      if !content.load_blocks.is_empty() {
        load.insert(
          "blocks".into(),
          serde_json::to_value(content.load_blocks).expect("serializable block IDs"),
        );
      }
      manifest_json["load"] = load.into();
    }
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

impl Plugin for HotModuleReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.HotModuleReplacementPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
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
  updated_chunk_ids: ChunkIdSet,
  removed_chunk_ids: ChunkIdSet,
  removed_modules: HashSet<ModuleId>,
  load_entries: BTreeMap<ChunkId, Vec<ChunkId>>,
  load_blocks: BlockChunks,
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
