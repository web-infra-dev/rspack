use std::path::Path;

use rspack_cacheable::{cacheable, with::AsMap};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  Content, LoaderContext, LoaderDependencies, LoaderItem, LoaderRunnerOptions, ParseMeta,
};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_sources::SourceMap;
use rspack_util::time::current_time;

use crate::{
  CacheOptions, CacheRecord, CacheValue, Etag, FileSystemInfo, IsolatedDts, Module, RscMeta,
  RunnerContext,
  new_cache::{CacheKey, Snapshot, SnapshotValidationResult},
};

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

fn sorted_dependency_paths(paths: &InternedPathSet) -> Vec<&Path> {
  let mut paths = paths.iter().map(|path| path.as_path()).collect::<Vec<_>>();
  paths.sort_unstable();
  paths
}

async fn loader_cache_etag(
  content: &Content,
  existing: &LoaderDependencies,
  loaders: &[LoaderItem<RunnerContext>],
  context: &RunnerContext,
) -> Option<Etag> {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  // Context and missing dependencies intentionally invalidate the minimal cache: inherited values
  // disable lookup, and entries that add either kind are skipped at store time. This trade-off lets
  // the etag omit both kinds entirely.
  // Equal bytes are not equivalent inputs: non-raw JS loaders strip a BOM only from buffers.
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content_is_string" => !content.is_buffer(),
    "content" => content,
    "file_dependencies" => sorted_dependency_paths(&existing.file),
    "build_dependencies" => sorted_dependency_paths(&existing.build),
    "rspack_version" => rspack_workspace::rspack_pkg_version!(),
    "source_map" => context.source_map_kind.source_map(),
  });
  for loader in loaders {
    let loader_build_dependencies = loader.loader().build_dependencies();
    let loader_build_dependencies_hash = if loader_build_dependencies.is_empty() {
      None
    } else {
      Some(
        context
          .file_system_info
          .build_dependencies_hash(loader_build_dependencies)
          .await
          .ok()?,
      )
    };
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
  file_dependencies: Snapshot,
  file_paths: Vec<InternedPath>,
  build_paths: Vec<InternedPath>,
  build_dependencies_hash: Option<u64>,
}

async fn loader_cache_dependency_snapshot(
  file_system_info: &FileSystemInfo,
  dependencies: &LoaderDependencies,
) -> Option<LoaderCacheDependencySnapshot> {
  if !dependencies.context.is_empty() || !dependencies.missing.is_empty() {
    return None;
  }
  let file_paths = dependencies.file.iter().cloned().collect::<Vec<_>>();
  let empty = InternedPathSet::default();
  let file_dependencies = file_system_info
    .create_snapshot(
      Some(current_time()),
      &dependencies.file,
      &empty,
      &empty,
      file_system_info.module_strategy(),
    )
    .await
    .ok()?;
  let mut build_paths = dependencies.build.iter().cloned().collect::<Vec<_>>();
  build_paths.sort_unstable_by(|a, b| a.as_path().cmp(b.as_path()));
  let build_dependencies_hash = if build_paths.is_empty() {
    None
  } else {
    Some(
      file_system_info
        .build_dependencies_hash(&build_paths)
        .await
        .ok()?,
    )
  };
  Some(LoaderCacheDependencySnapshot {
    file_dependencies,
    file_paths,
    build_paths,
    build_dependencies_hash,
  })
}

async fn loader_cache_dependency_snapshot_is_valid(
  file_system_info: &FileSystemInfo,
  snapshot: &LoaderCacheDependencySnapshot,
) -> bool {
  if !matches!(
    file_system_info
      .check_snapshot_valid(&snapshot.file_dependencies)
      .await,
    Ok(SnapshotValidationResult::Valid)
  ) {
    return false;
  }
  if snapshot.build_paths.is_empty() {
    return snapshot.build_dependencies_hash.is_none();
  }
  let Some(expected_hash) = snapshot.build_dependencies_hash else {
    return false;
  };
  matches!(
    file_system_info
      .build_dependencies_hash(&snapshot.build_paths)
      .await,
    Ok(current_hash) if current_hash == expected_hash
  )
}

fn restore_loader_cache_dependencies(
  snapshot: &LoaderCacheDependencySnapshot,
  dependencies: &mut LoaderDependencies,
) {
  dependencies
    .file
    .extend(snapshot.file_paths.iter().cloned());
  dependencies
    .build
    .extend(snapshot.build_paths.iter().cloned());
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
  let Some(loader_cache) = &context.context.loader_cache else {
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
    || !existing_dependencies.context.is_empty()
    || !existing_dependencies.missing.is_empty()
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
  let Some(etag) =
    loader_cache_etag(content, existing_dependencies, loaders, &context.context).await
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
    let mut dependencies = LoaderDependencies::default();
    restore_loader_cache_dependencies(&entry.dependency_snapshot, &mut dependencies);
    context
      .context
      .loader_cache_counters
      .record(CacheRecord::Reuse);
    context.add_dependencies(&dependencies);
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
    diagnostics_len: context.diagnostics.len(),
  })))
}

pub(crate) async fn after_normal_chain(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) {
  let Some(loader_cache) = &context.context.loader_cache else {
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
    context.added_dependencies(),
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
