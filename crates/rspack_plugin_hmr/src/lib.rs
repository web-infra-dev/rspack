mod hot_module_replacement;
mod initial_chunks;

use std::{
  collections::{BTreeMap, hash_map},
  sync::{Arc, LazyLock, Mutex},
};

use atomic_refcell::AtomicRefCell;
use hot_module_replacement::HotModuleReplacementRuntimeModule;
use initial_chunks::{EntryChunks, collect_entry_chunks};
use rspack_collections::IdentifierSet;
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAsset, CompilationContentHash,
  CompilationParams, CompilationProcessAssets, CompilationRecords, CompilerCompilation,
  DependencyType, LoaderContext, ManifestAssetType, ModuleId, ModuleIdentifier, ModuleType,
  NormalModuleFactoryParser, NormalModuleLoader, ParserAndGenerator, ParserOptions, PathData,
  Plugin, RunnerContext, RuntimeGlobals, RuntimeModule, RuntimeModuleExt, RuntimeSpec, SourceType,
  chunk_graph_chunk::{ChunkId, ChunkIdMap, ChunkIdSet},
  chunk_graph_module::ModuleIdSet,
  incremental::{IncrementalPasses, Mutation},
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
use rspack_plugin_runtime::get_hmr_chunk_snapshot_hash;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

type AsyncChunkGroups = BTreeMap<String, Vec<ChunkId>>;

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// The baseline snapshots are only touched from the processAssets hook, which
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
  previous_css_files: AtomicRefCell<Arc<ChunkIdMap<ChunkCssFiles>>>,
  // Keep the last recorded build and the current candidate. A failed build
  // may not advance CompilationRecords, so key these snapshots by its hash.
  entry_chunks: AtomicRefCell<HashMap<RspackHashDigest, Arc<EntryChunks>>>,
  async_chunks: AtomicRefCell<HashMap<RspackHashDigest, Arc<AsyncChunkGroups>>>,
  // Retain immutable normal-chunk versions while clients catch up with HMR.
  chunk_snapshots: AtomicRefCell<HashMap<String, CompilationAsset>>,
}

impl Default for HotModuleReplacementPlugin {
  fn default() -> Self {
    Self::new_inner(
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
    )
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

/// Filenames needed while preparing CSS updates before the filename runtime changes.
#[derive(Debug, Default)]
struct ChunkCssFiles {
  css: Option<String>,
  mini_css: Option<String>,
}

fn collect_css_files(compilation: &Compilation) -> ChunkIdMap<ChunkCssFiles> {
  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
    .filter_map(|chunk| {
      let mut files = ChunkCssFiles::default();
      for filename in chunk.files() {
        if let Some(asset) = compilation.assets().get(filename) {
          match &asset.info.asset_type {
            ManifestAssetType::Css => files.css = Some(filename.clone()),
            ManifestAssetType::Custom(kind) if kind.as_str() == "extract-css" => {
              files.mini_css = Some(filename.clone())
            }
            _ => {}
          }
        }
      }
      (files.css.is_some() || files.mini_css.is_some()).then(|| (chunk.expect_id().clone(), files))
    })
    .collect()
}

impl ChunkCssHashes {
  fn is_empty(&self) -> bool {
    self.css.is_none() && self.mini_css.is_none()
  }
}

#[plugin_hook(CompilerCompilation for HotModuleReplacementPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.hot_module_replacement = true;
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
  let current_css_files = Arc::new(collect_css_files(compilation));
  let old_css_files = std::mem::replace(
    &mut *self.previous_css_files.borrow_mut(),
    Arc::clone(&current_css_files),
  );

  let current_entries = Arc::new(collect_entry_chunks(compilation));
  let current_groups = Arc::new(
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_async_chunk_groups(compilation, None),
  );
  let old_groups = {
    let old_hash = compilation
      .records
      .as_ref()
      .and_then(|records| records.hash.as_ref());
    let mut snapshots = self.async_chunks.borrow_mut();
    snapshots.retain(|hash, _| Some(hash) == old_hash);
    let previous = old_hash
      .and_then(|hash| snapshots.get(hash))
      .cloned()
      .unwrap_or_default();
    if let Some(hash) = &compilation.hash {
      snapshots.insert(hash.clone(), Arc::clone(&current_groups));
    }
    previous
  };
  let old_entries = {
    let old_hash = compilation
      .records
      .as_ref()
      .and_then(|records| records.hash.as_ref());
    let mut snapshots = self.entry_chunks.borrow_mut();
    snapshots.retain(|hash, _| Some(hash) == old_hash);
    let previous = old_hash
      .and_then(|hash| snapshots.get(hash))
      .cloned()
      .unwrap_or_default();
    if let Some(hash) = &compilation.hash {
      snapshots.insert(hash.clone(), Arc::clone(&current_entries));
    }
    previous
  };

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

  // A runtime can be shared by pages with different entrypoints. Use the old
  // entry chunk as the loaded-page anchor, rather than treating every initial
  // chunk in the compilation as a dependency of every page.
  let mut initial_chunks = ChunkIdSet::default();
  let mut entry_names = current_entries.keys().collect::<Vec<_>>();
  entry_names.sort_unstable();
  for name in entry_names {
    let (entry_id, dependencies) = &current_entries[name];
    let Some((old_entry_id, old_dependencies)) = old_entries.get(name) else {
      continue;
    };
    let Some(entry_ukey) = current_chunk_ukeys.get(entry_id) else {
      continue;
    };
    let entry = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(entry_ukey);
    let added: Vec<_> = dependencies
      .iter()
      .filter(|id| !old_dependencies.contains(id))
      .cloned()
      .collect();
    if added.is_empty() {
      continue;
    }
    initial_chunks.extend(added.iter().cloned());
    for runtime in entry.runtime().iter() {
      if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
        info.load_chunks.insert(old_entry_id.clone(), added.clone());
      }
    }
  }
  for (id, chunks) in current_groups.iter() {
    let Some(old) = old_groups.get(id) else {
      continue;
    };
    let added: Vec<_> = chunks
      .iter()
      .filter(|chunk| !old.contains(chunk))
      .cloned()
      .collect();
    if added.is_empty() {
      continue;
    }
    initial_chunks.extend(added.iter().cloned());
    for info in hot_update_main_content_by_runtime.values_mut() {
      info.group_chunks.insert(id.clone(), added.clone());
    }
  }
  let chunk_ids: Vec<_> = old_chunks
    .keys()
    .chain(
      initial_chunks
        .iter()
        .filter(|id| !old_chunks.contains_key(*id)),
    )
    .cloned()
    .collect();
  let empty_runtime = RuntimeSpec::default();
  let empty_module_ids = ModuleIdSet::default();
  for chunk_id in chunk_ids {
    let (old_runtime, old_module_ids) = old_chunks
      .get(&chunk_id)
      .map_or((&empty_runtime, &empty_module_ids), |(runtime, modules)| {
        (runtime, modules)
      });
    let full_chunk = initial_chunks.contains(&chunk_id);
    let mut new_modules = vec![];
    let mut new_runtime_modules = vec![];
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

      if !full_chunk
        && old_runtime == &new_runtime
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
    let old_files = old_css_files.get(&chunk_id);
    let new_files = current_css_files.get(&chunk_id);

    for removed in removed_from_runtime.iter() {
      if let Some(info) = hot_update_main_content_by_runtime.get_mut(removed) {
        info.removed_chunk_ids.insert(chunk_id.clone());
        if old_css_hashes.is_some_and(|hashes| hashes.css.is_some()) {
          info.css_removed_chunk_ids.insert(chunk_id.clone());
          if let Some(filename) = old_files.and_then(|files| files.css.as_ref()) {
            info
              .css_old_files
              .insert(chunk_id.clone(), filename.clone());
          }
        }
        if old_css_hashes.is_some_and(|hashes| hashes.mini_css.is_some()) {
          info.mini_css_removed_chunk_ids.insert(chunk_id.clone());
          if let Some(filename) = old_files.and_then(|files| files.mini_css.as_ref()) {
            info
              .mini_css_old_files
              .insert(chunk_id.clone(), filename.clone());
          }
        }
      }
    }

    // Independent of whether the chunk carries updated js modules: a chunk
    // holding only extracted css has no js update when its stylesheet changes.
    if current_chunk.is_some() {
      for runtime in new_runtime.iter() {
        if let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime) {
          if css_update != CssUpdate::Unchanged || full_chunk {
            if let Some(filename) = old_files.and_then(|files| files.css.as_ref()) {
              info
                .css_old_files
                .insert(chunk_id.clone(), filename.clone());
            }
            if let Some(filename) = new_files.and_then(|files| files.css.as_ref()) {
              info.css_files.insert(chunk_id.clone(), filename.clone());
            }
          }
          if mini_css_update != CssUpdate::Unchanged || full_chunk {
            if let Some(filename) = old_files.and_then(|files| files.mini_css.as_ref()) {
              info
                .mini_css_old_files
                .insert(chunk_id.clone(), filename.clone());
            }
            if let Some(filename) = new_files.and_then(|files| files.mini_css.as_ref()) {
              info
                .mini_css_files
                .insert(chunk_id.clone(), filename.clone());
            }
          }
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
          if let Some(content) = hot_update_main_content_by_runtime.get_mut(removed) {
            content.removed_modules.insert(old_module_id.clone());
          }
        }
      }
    }

    let has_regular_update = !new_modules.is_empty() || !new_runtime_modules.is_empty();
    if full_chunk && let Some(current_chunk) = current_chunk {
      new_modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(&current_chunk.ukey())
        .iter()
        .copied()
        .collect();
    }

    if !new_modules.is_empty() || !new_runtime_modules.is_empty() {
      // A factory can appear in another chunk without changing. Keep its
      // existing instance, whether the payload is supplemental or incremental.
      for module in &new_modules {
        let Some(module_id) = ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module)
        else {
          continue;
        };
        let Some(old_hashes) = old_all_modules.get(module_id) else {
          continue;
        };
        let current_runtime = current_chunk.map_or(&new_runtime, |chunk| chunk.runtime());
        let Some(new_hash) = compilation
          .code_generation_results
          .get_hash(module, Some(current_runtime))
        else {
          continue;
        };
        for runtime in new_runtime.iter() {
          let mut previous = old_hashes
            .iter()
            .filter(|(id, _)| {
              old_chunks
                .get(*id)
                .is_some_and(|(runtimes, _)| runtimes.contains(runtime))
            })
            .peekable();
          if previous.peek().is_some()
            && previous.all(|(_, hash)| hash == new_hash)
            && let Some(info) = hot_update_main_content_by_runtime.get_mut(runtime)
          {
            info.unchanged_modules.insert(module_id.clone());
          }
        }
      }
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
          // Supplement-only payloads are requested through `load`. Keep a
          // regular update for runtimes that could already have this chunk.
          if !full_chunk || (has_regular_update && old_runtime.contains(runtime)) {
            info.updated_chunk_ids.insert(chunk_id.clone());
          }
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
        old_content.load_chunks.extend(content.load_chunks);
        old_content.group_chunks.extend(content.group_chunks);
        old_content
          .unchanged_modules
          .extend(content.unchanged_modules);
        old_content.css_files.extend(content.css_files);
        old_content.css_old_files.extend(content.css_old_files);
        old_content.mini_css_files.extend(content.mini_css_files);
        old_content
          .mini_css_old_files
          .extend(content.mini_css_old_files);
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
    if !content.load_chunks.is_empty() {
      manifest_json["load"] = serde_json::json!(content.load_chunks);
    }
    if !content.group_chunks.is_empty() {
      manifest_json["groups"] = serde_json::json!(content.group_chunks);
    }
    if !content.unchanged_modules.is_empty() {
      manifest_json["unchanged"] = serde_json::json!(content.unchanged_modules);
    }
    if let Some(css) = css_manifest_json(
      content.css_updated_chunk_ids,
      content.css_removed_chunk_ids,
      content.css_files,
      content.css_old_files,
    ) {
      manifest_json["css"] = css;
    }
    if let Some(mini_css) = css_manifest_json(
      content.mini_css_updated_chunk_ids,
      content.mini_css_removed_chunk_ids,
      content.mini_css_files,
      content.mini_css_old_files,
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

#[plugin_hook(CompilationProcessAssets for HotModuleReplacementPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_AFTER_OPTIMIZE_HASH)]
async fn chunk_snapshots(&self, compilation: &mut Compilation) -> Result<()> {
  let mut snapshots = self.chunk_snapshots.borrow_mut();
  for chunk in compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
  {
    let Some(hash) = get_hmr_chunk_snapshot_hash(chunk, compilation) else {
      continue;
    };
    for filename in chunk.files() {
      let Some(asset) = compilation.assets().get(filename) else {
        continue;
      };
      if asset.info.asset_type != ManifestAssetType::JavaScript {
        continue;
      }
      let end = filename.find(['?', '#']).unwrap_or(filename.len());
      let directory = filename[..end].rfind('/').map_or(0, |index| index + 1);
      let snapshot_filename = format!(
        "{}{}.hot-chunk.{}",
        &filename[..directory],
        hash,
        &filename[directory..]
      );
      snapshots.entry(snapshot_filename).or_insert_with(|| {
        let mut info = asset.get_info().clone();
        info.immutable = Some(true);
        info.development = Some(true);
        info.hot_module_replacement = Some(true);
        CompilationAsset::new(asset.source.clone(), info)
      });
    }
  }
  for (filename, asset) in snapshots.iter() {
    compilation.emit_asset(filename.clone(), asset.clone());
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
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(chunk_snapshots::new(self));
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
  css_updated_chunk_ids: ChunkIdSet,
  css_removed_chunk_ids: ChunkIdSet,
  mini_css_updated_chunk_ids: ChunkIdSet,
  mini_css_removed_chunk_ids: ChunkIdSet,
  load_chunks: ChunkIdMap<Vec<ChunkId>>,
  group_chunks: AsyncChunkGroups,
  unchanged_modules: HashSet<ModuleId>,
  css_files: ChunkIdMap<String>,
  css_old_files: ChunkIdMap<String>,
  mini_css_files: ChunkIdMap<String>,
  mini_css_old_files: ChunkIdMap<String>,
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

fn css_manifest_json(
  updated: ChunkIdSet,
  removed: ChunkIdSet,
  files: ChunkIdMap<String>,
  old_files: ChunkIdMap<String>,
) -> Option<serde_json::Value> {
  if updated.is_empty() && removed.is_empty() && files.is_empty() {
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
  // Serialize filename lookup tables deterministically.
  if !files.is_empty() {
    css.insert(
      "f".to_string(),
      serde_json::json!(files.into_iter().collect::<BTreeMap<_, _>>()),
    );
  }
  if !old_files.is_empty() {
    css.insert(
      "o".to_string(),
      serde_json::json!(old_files.into_iter().collect::<BTreeMap<_, _>>()),
    );
  }
  Some(serde_json::Value::Object(css))
}
