use std::path::Path;

use bitflags::bitflags;
use rspack_cacheable::{cacheable, with::AsMap};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  Content, LoaderChain, LoaderContext, LoaderDependencies, LoaderRunnerOptions, ParseMeta,
};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_sources::SourceMap;
use rspack_util::time::current_time;

use crate::{
  CacheFacade, CacheValue, Etag, FileSystemInfo, IsolatedDts, ItemCacheFacade, Module, RscMeta,
  RunnerContext,
  cache::SnapshotStrategyOptions,
  new_cache::{Snapshot, SnapshotValidationResult},
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
      "options" => &options.options_cache_key,
      "loader_version" => &options.loader_version,
    });
  }
  Some(format!("{:016x}", hasher.finish()))
}

fn sorted_dependency_paths(paths: &InternedPathSet) -> Vec<&Path> {
  let mut paths = paths.iter().map(|path| path.as_path()).collect::<Vec<_>>();
  paths.sort_unstable();
  paths
}

fn loader_cache_etag(content: &Content, existing: &LoaderDependencies) -> Etag {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  // Context and missing dependencies intentionally invalidate the minimal cache: inherited values
  // disable lookup, and entries that add either kind are skipped at store time. This trade-off lets
  // the etag omit both kinds entirely.
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content" => content,
    "file_dependencies" => sorted_dependency_paths(&existing.file),
    "build_dependencies" => sorted_dependency_paths(&existing.build),
    "rspack_version" => rspack_workspace::rspack_pkg_version!(),
  });
  Etag::from(format!("{:016x}", hasher.finish()))
}

#[cacheable]
#[derive(Clone, Copy)]
struct LoaderCacheDependencyKind(u8);

bitflags! {
  impl LoaderCacheDependencyKind: u8 {
    const FILE = 1 << 0;
    const BUILD = 1 << 1;
  }
}

#[cacheable]
struct LoaderCacheDependencySnapshot {
  dependencies: Snapshot,
  paths: Vec<InternedPath>,
  kinds: Vec<LoaderCacheDependencyKind>,
}

async fn loader_cache_dependency_snapshot(
  file_system_info: &FileSystemInfo,
  dependencies: &LoaderDependencies,
) -> Option<LoaderCacheDependencySnapshot> {
  if !dependencies.context.is_empty() || !dependencies.missing.is_empty() {
    return None;
  }
  let mut files = InternedPathSet::default();
  let mut paths = Vec::with_capacity(dependencies.file.len() + dependencies.build.len());
  let mut kinds = Vec::with_capacity(paths.capacity());
  for path in dependencies.file.union(&dependencies.build) {
    files.insert(path.clone());
    paths.push(path.clone());
    let mut kind = LoaderCacheDependencyKind::empty();
    if dependencies.file.contains(path) {
      kind.insert(LoaderCacheDependencyKind::FILE);
    }
    if dependencies.build.contains(path) {
      kind.insert(LoaderCacheDependencyKind::BUILD);
    }
    debug_assert!(!kind.is_empty());
    kinds.push(kind);
  }
  let empty = InternedPathSet::default();
  let dependencies = file_system_info
    .create_snapshot(
      Some(current_time()),
      &files,
      &empty,
      &empty,
      SnapshotStrategyOptions::timestamp(),
    )
    .await
    .ok()?;
  Some(LoaderCacheDependencySnapshot {
    dependencies,
    paths,
    kinds,
  })
}

async fn loader_cache_dependency_snapshot_is_valid(
  file_system_info: &FileSystemInfo,
  snapshot: &LoaderCacheDependencySnapshot,
) -> bool {
  snapshot.paths.len() == snapshot.kinds.len()
    && matches!(
      file_system_info
        .check_snapshot_valid(&snapshot.dependencies)
        .await,
      Ok(SnapshotValidationResult::Valid)
    )
}

fn restore_loader_cache_dependencies(
  snapshot: &LoaderCacheDependencySnapshot,
  dependencies: &mut LoaderDependencies,
) {
  for (path, kind) in snapshot.paths.iter().zip(&snapshot.kinds) {
    if kind.contains(LoaderCacheDependencyKind::FILE) {
      dependencies.file.insert(path.clone());
    }
    if kind.contains(LoaderCacheDependencyKind::BUILD) {
      dependencies.build.insert(path.clone());
    }
  }
}

fn loader_cache_item(storage: &CacheFacade, key: &str, etag: Etag) -> ItemCacheFacade {
  storage.get_item_cache(key, Some(etag))
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
  rsc: Option<RscMeta>,
}

pub(crate) struct LoaderCacheMissState {
  cache_key: String,
  etag: Etag,
  diagnostics_len: usize,
  existing_dependencies: LoaderDependencies,
}

pub(crate) enum LoaderCacheAction {
  Disabled,
  Hit,
  Miss(Box<LoaderCacheMissState>),
}

fn input_etag(context: &LoaderContext<RunnerContext>) -> Option<Etag> {
  Some(loader_cache_etag(
    context.content()?,
    context.existing_dependencies(),
  ))
}

pub(crate) async fn before_normal_chain(
  context: &mut LoaderContext<RunnerContext>,
  chain: &LoaderChain,
) -> Result<LoaderCacheAction> {
  debug_assert!(chain.is_cache());
  if !context.cacheable {
    return Ok(LoaderCacheAction::Disabled);
  }
  // Source maps are intentionally excluded from the etag as a performance trade-off. The minimal
  // cache treats source-map-only changes as equivalent inputs.
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
  let Some(etag) = input_etag(context) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let module_identifier = context.context.module.identifier();
  let Some(cache_key) = loader_cache_key(
    module_identifier.as_str(),
    chain
      .range()
      .map(|index| context.loader_items()[usize::from(index)].cache_options()),
  ) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let item_cache = loader_cache_item(&context.context.loader_cache, &cache_key, etag.clone());

  if let Some(entry) = item_cache.get::<LoaderCacheEntry>()?
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
    existing_dependencies: context.existing_dependencies().clone(),
  })))
}

pub(crate) async fn after_normal_chain(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.additional_data().is_some()
  {
    return;
  }

  let existing = &state.existing_dependencies;
  let current = context.dependencies();
  if !existing.is_subset_of(&current) {
    return;
  }
  let added_dependencies = current.difference(existing);
  let Some(dependency_snapshot) =
    loader_cache_dependency_snapshot(&context.context.file_system_info, &added_dependencies).await
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
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    &state.cache_key,
    state.etag.clone(),
  );
  item_cache.store(CacheValue::new(entry));
}
