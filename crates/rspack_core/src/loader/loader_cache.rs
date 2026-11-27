use std::sync::Arc;

use rspack_cacheable::{cacheable, with::AsMap};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher, UnorderedHasher};
use rspack_loader_runner::{
  Content, LoaderContext, LoaderDependencies, LoaderItem, LoaderRunnerOptions, ParseMeta,
};
use rspack_paths::{InternedPath, InternedPathDashMap};
use rspack_sources::SourceMap;
use rspack_util::time::current_time;
use tokio::sync::OnceCell;

use crate::{
  CacheOptions, CacheRecord, CacheValue, Etag, FileSystemInfo, IsolatedDts, Module, RscMeta,
  RunnerContext,
  new_cache::{CacheKey, Snapshot, SnapshotValidationResult},
};

/// Per-compilation cache for hashing resolved loader build dependencies.
///
/// Only allocated when `loader_cache` is enabled so disabled builds pay no overhead.
#[derive(Debug, Default)]
pub struct LoaderBuildDependenciesCache {
  hashes: InternedPathDashMap<Arc<OnceCell<u64>>>,
}

impl LoaderBuildDependenciesCache {
  /// Resolves and hashes the contents of build dependency paths once per compilation.
  pub(crate) async fn build_dependencies_hash(
    &self,
    file_system_info: &FileSystemInfo,
    paths: &mut (dyn Iterator<Item = &InternedPath> + Send),
  ) -> Result<Option<u64>> {
    let mut unordered = UnorderedHasher::new();
    for path in paths {
      let path_hash = self.build_dependency_hash(file_system_info, path).await?;
      unordered.write_entry(path, &path_hash);
    }
    if unordered.is_empty() {
      return Ok(None);
    }
    let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
    unordered.finish_map(&mut hasher);
    Ok(Some(hasher.finish()))
  }

  async fn build_dependency_hash(
    &self,
    file_system_info: &FileSystemInfo,
    path: &InternedPath,
  ) -> Result<u64> {
    let cell = if let Some(cell) = self.hashes.get(path) {
      Arc::clone(cell.value())
    } else {
      Arc::clone(
        self
          .hashes
          .entry(path.clone())
          .or_insert_with(|| Arc::new(OnceCell::new()))
          .value(),
      )
    };
    cell
      .get_or_try_init(|| self.compute_build_dependency_hash(file_system_info, path))
      .await
      .copied()
  }

  async fn compute_build_dependency_hash(
    &self,
    file_system_info: &FileSystemInfo,
    path: &InternedPath,
  ) -> Result<u64> {
    let resolved = file_system_info
      .resolve_build_dependencies(std::iter::once(path.clone()))
      .await;
    let mut strategy = file_system_info.build_dependencies_strategy();
    if strategy.hash {
      // Folding a temporary snapshot into an etag cannot use the timestamp fast-path
      // without invalidating on mtime-only changes; skip timestamp capture when hash is enabled.
      strategy.timestamp = false;
    }
    let snapshot = file_system_info
      .create_snapshot(
        None,
        &resolved.files,
        &resolved.contexts,
        &resolved.missing,
        strategy,
      )
      .await?;
    Ok(snapshot.fingerprint())
  }
}

fn loader_cache_key<'a>(
  module_identifier: &str,
  loader_options: impl Iterator<Item = Option<&'a LoaderRunnerOptions>>,
) -> Option<String> {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "module" => module_identifier,
  });
  for options in loader_options {
    let options = options?;
    rspack_hash::rspack_hash_object!(&mut hasher, {
      "loader" => &options.loader_name,
    });
  }
  Some(format!("{:016x}", hasher.finish()))
}

async fn loader_cache_etag(
  content: &Content,
  existing: &LoaderDependencies,
  loaders: &[LoaderItem<RunnerContext>],
  context: &RunnerContext,
  build_dependencies_cache: &LoaderBuildDependenciesCache,
) -> Option<Etag> {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  // Equal bytes are not equivalent inputs: non-raw JS loaders strip a BOM only from buffers.
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content_is_string" => !content.is_buffer(),
    "content" => content,
    "file_dependencies" => &existing.file,
    "context_dependencies" => &existing.context,
    "missing_dependencies" => &existing.missing,
    "build_dependencies" => &existing.build,
    "rspack_version" => rspack_workspace::rspack_pkg_version!(),
    "source_map" => context.source_map_kind.source_map(),
  });
  for loader in loaders {
    let loader_build_dependencies_hash = build_dependencies_cache
      .build_dependencies_hash(
        &context.file_system_info,
        &mut loader.loader().build_dependencies().iter(),
      )
      .await
      .ok()?;
    rspack_hash::rspack_hash_object!(&mut hasher, {
      "rule_loader_options" => loader.options_cache_key(),
      "loader_version" => loader.loader_version(),
      "has_loader_build_dependencies" => loader_build_dependencies_hash.is_some(),
      "loader_build_dependencies_hash" => loader_build_dependencies_hash,
    });
    loader.loader().etag(context, &mut hasher);
  }
  Some(Etag::from(format!("{:016x}", hasher.finish())))
}

#[cacheable]
struct LoaderCacheDependencySnapshot {
  dependencies: LoaderDependencies,
  snapshot: Snapshot,
  build_dependencies_hash: Option<u64>,
}

async fn loader_cache_dependency_snapshot(
  file_system_info: &FileSystemInfo,
  build_dependencies_cache: &LoaderBuildDependenciesCache,
  dependencies: &LoaderDependencies,
  start_time: u64,
) -> Option<LoaderCacheDependencySnapshot> {
  let snapshot = file_system_info
    .create_snapshot(
      Some(start_time),
      &dependencies.file,
      &dependencies.context,
      &dependencies.missing,
      file_system_info.module_strategy(),
    )
    .await
    .ok()?;
  let build_dependencies_hash = build_dependencies_cache
    .build_dependencies_hash(file_system_info, &mut dependencies.build.iter())
    .await
    .ok()?;
  Some(LoaderCacheDependencySnapshot {
    dependencies: dependencies.clone(),
    snapshot,
    build_dependencies_hash,
  })
}

async fn loader_cache_dependency_snapshot_is_valid(
  file_system_info: &FileSystemInfo,
  build_dependencies_cache: &LoaderBuildDependenciesCache,
  snapshot: &LoaderCacheDependencySnapshot,
) -> bool {
  if !matches!(
    file_system_info
      .check_snapshot_valid(&snapshot.snapshot)
      .await,
    Ok(SnapshotValidationResult::Valid)
  ) {
    return false;
  }
  matches!(
    build_dependencies_cache
      .build_dependencies_hash(file_system_info, &mut snapshot.dependencies.build.iter())
      .await,
    Ok(current_hash) if current_hash == snapshot.build_dependencies_hash
  )
}

#[cacheable]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<String>,
  dependency_snapshot: LoaderCacheDependencySnapshot,
  #[cacheable(with=AsMap)]
  parse_meta: ParseMeta,
  isolated_dts: Option<Box<IsolatedDts>>,
  rsc: Option<Box<RscMeta>>,
}

pub(crate) struct LoaderCacheMissState {
  cache_key: CacheKey,
  etag: Etag,
  start_time: u64,
  diagnostics_len: usize,
}

pub(crate) enum LoaderCacheAction {
  Disabled,
  Hit,
  Miss(Box<LoaderCacheMissState>),
}

pub(crate) async fn before_normal_chain(
  context: &mut LoaderContext<RunnerContext>,
) -> Result<LoaderCacheAction> {
  let (Some(loader_cache), Some(build_dependencies_cache)) = (
    &context.context.loader_cache,
    &context.context.loader_build_dependencies_cache,
  ) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let chain = context
    .current_root_chain()
    .expect("normal execution requires a current root chain");
  debug_assert!(chain.is_cache());
  if !context.cacheable {
    return Ok(LoaderCacheAction::Disabled);
  }
  let existing_dependencies = context.existing_dependencies();
  if context.additional_data().is_some()
    || !context.parse_meta.is_empty()
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.context.module.build_info().isolated_dts.is_some()
    || context.context.module.build_info().rsc.is_some()
  {
    return Ok(LoaderCacheAction::Disabled);
  }
  let Some(content) = context.content() else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let range = chain.range();
  let loaders = &context.loader_items[usize::from(range.start)..usize::from(range.end)];
  let module_identifier = context.context.module.identifier();
  let Some(cache_key) = loader_cache_key(
    module_identifier.as_str(),
    loaders.iter().map(|loader| loader.cache_options()),
  ) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let Some(etag) = loader_cache_etag(
    content,
    existing_dependencies,
    loaders,
    &context.context,
    build_dependencies_cache,
  )
  .await
  else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let cache_key = CacheKey::from(cache_key);
  let entry = loader_cache.get::<LoaderCacheEntry>(cache_key.clone(), Some(etag.clone()));
  context
    .context
    .loader_cache_counters
    .record(if entry.is_some() {
      CacheRecord::Hit
    } else {
      CacheRecord::Miss
    });
  if let Some(entry) = entry
    && loader_cache_dependency_snapshot_is_valid(
      &context.context.file_system_info,
      build_dependencies_cache,
      &entry.dependency_snapshot,
    )
    .await
  {
    let content = match (&entry.content, entry.content_is_string) {
      (Some(content), true) => {
        // SAFETY: String cache entries are written exclusively from `Content::String`.
        let content = unsafe { String::from_utf8_unchecked(content.clone()) };
        Some(Content::String(content))
      }
      (Some(content), false) => Some(Content::Buffer(content.clone())),
      (None, _) => None,
    };
    let source_map = entry
      .source_map
      .clone()
      .and_then(|source_map| SourceMap::from_json(source_map).ok());
    context
      .context
      .loader_cache_counters
      .record(CacheRecord::Reuse);
    context.add_dependencies(&entry.dependency_snapshot.dependencies);
    context.parse_meta = entry.parse_meta.clone();
    let build_info = context.context.module.build_info_mut();
    build_info.isolated_dts = entry.isolated_dts.clone();
    build_info.rsc = entry.rsc.clone();
    context.__finish_with((content, source_map, None));
    return Ok(LoaderCacheAction::Hit);
  }

  Ok(LoaderCacheAction::Miss(Box::new(LoaderCacheMissState {
    cache_key,
    etag,
    start_time: current_time(),
    diagnostics_len: context.diagnostics.len(),
  })))
}

pub(crate) async fn after_normal_chain(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) {
  let (Some(loader_cache), Some(build_dependencies_cache)) = (
    &context.context.loader_cache,
    &context.context.loader_build_dependencies_cache,
  ) else {
    return;
  };
  if matches!(&context.context.options.cache, CacheOptions::Persistent(options) if options.readonly)
    || !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.additional_data().is_some()
  {
    return;
  }

  if !context.removed_dependencies().is_empty() {
    return;
  }
  let Some(dependency_snapshot) = loader_cache_dependency_snapshot(
    &context.context.file_system_info,
    build_dependencies_cache,
    context.added_dependencies(),
    state.start_time,
  )
  .await
  else {
    return;
  };

  let (content, content_is_string) = match context.content() {
    Some(Content::String(content)) => (Some(content.as_bytes().to_vec()), true),
    Some(Content::Buffer(content)) => (Some(content.clone()), false),
    None => (None, false),
  };
  let entry = LoaderCacheEntry {
    content,
    content_is_string,
    source_map: context.source_map().map(SourceMap::to_json),
    dependency_snapshot,
    parse_meta: context.parse_meta.clone(),
    isolated_dts: context.context.module.build_info().isolated_dts.clone(),
    rsc: context.context.module.build_info().rsc.clone(),
  };
  context
    .context
    .loader_cache_counters
    .record(CacheRecord::Store);
  loader_cache.store(
    state.cache_key.clone(),
    Some(state.etag.clone()),
    CacheValue::new(entry),
  );
}
