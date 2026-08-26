use bitflags::bitflags;
use rspack_cacheable::cacheable;
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{Content, LoaderContext, LoaderDependencyContext};
use rspack_sources::SourceMap;

use crate::{
  CacheFacade, CacheValue, Etag, ItemCacheFacade, Module, RunnerContext,
  new_cache::FileDependencies,
};

fn loader_cache_key(module_identifier: &str, loader_name: &str) -> String {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "module" => module_identifier,
    "loader" => loader_name,
  });
  format!("{:016x}", hasher.finish())
}

#[doc(hidden)]
pub fn loader_cache_etag(content: &Content, options_cache_key: &str, loader_version: &str) -> Etag {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content" => content,
    "options" => options_cache_key,
    "loader_version" => loader_version,
    "rspack_version" => rspack_workspace::rspack_pkg_version!(),
  });
  Etag::from(format!("{:016x}", hasher.finish()))
}

#[doc(hidden)]
#[cacheable]
#[derive(Clone, Copy)]
struct LoaderCacheDependencyKind(u8);

bitflags! {
  impl LoaderCacheDependencyKind: u8 {
    const FILE = 1 << 0;
    const BUILD = 1 << 1;
  }
}

#[doc(hidden)]
#[cacheable]
#[derive(Clone)]
pub struct LoaderCacheDependencySnapshot {
  dependencies: FileDependencies,
  kinds: Vec<LoaderCacheDependencyKind>,
}

#[doc(hidden)]
pub fn loader_cache_dependency_snapshot(
  fs: &dyn ReadableFileSystem,
  dependency_context: &LoaderDependencyContext,
) -> Option<LoaderCacheDependencySnapshot> {
  if !dependency_context.context.is_empty() || !dependency_context.missing.is_empty() {
    return None;
  }
  let mut paths =
    Vec::with_capacity(dependency_context.file.len() + dependency_context.build.len());
  let mut kinds = Vec::with_capacity(paths.capacity());
  for path in dependency_context.file.union(&dependency_context.build) {
    paths.push(path.clone());
    let mut kind = LoaderCacheDependencyKind::empty();
    if dependency_context.file.contains(path) {
      kind.insert(LoaderCacheDependencyKind::FILE);
    }
    if dependency_context.build.contains(path) {
      kind.insert(LoaderCacheDependencyKind::BUILD);
    }
    debug_assert!(!kind.is_empty());
    kinds.push(kind);
  }
  let dependencies = FileDependencies::capture(fs, paths)?;
  Some(LoaderCacheDependencySnapshot {
    dependencies,
    kinds,
  })
}

#[doc(hidden)]
pub fn loader_cache_dependency_snapshot_is_valid(
  fs: &dyn ReadableFileSystem,
  snapshot: &LoaderCacheDependencySnapshot,
) -> bool {
  snapshot.dependencies.paths().len() == snapshot.kinds.len() && snapshot.dependencies.is_valid(fs)
}

#[doc(hidden)]
pub fn restore_loader_cache_dependencies(
  snapshot: &LoaderCacheDependencySnapshot,
  dependency_context: &mut LoaderDependencyContext,
) {
  for (path, kind) in snapshot.dependencies.paths().zip(&snapshot.kinds) {
    if kind.contains(LoaderCacheDependencyKind::FILE) {
      dependency_context.file.insert(path.clone());
    }
    if kind.contains(LoaderCacheDependencyKind::BUILD) {
      dependency_context.build.insert(path.clone());
    }
  }
}

#[doc(hidden)]
pub fn loader_cache_item(
  storage: &CacheFacade,
  module_identifier: &str,
  loader_name: &str,
  etag: Etag,
) -> ItemCacheFacade {
  let key = loader_cache_key(module_identifier, loader_name);
  storage.get_item_cache(&key, Some(etag))
}

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<String>,
  dependency_snapshot: LoaderCacheDependencySnapshot,
}

pub(crate) struct LoaderCacheMissState {
  etag: Etag,
  diagnostics_len: usize,
}

pub(crate) enum LoaderCacheAction {
  Disabled,
  Hit,
  Miss(Box<LoaderCacheMissState>),
}

fn cache_miss_action(context: &LoaderContext<RunnerContext>, etag: Etag) -> LoaderCacheAction {
  LoaderCacheAction::Miss(Box::new(LoaderCacheMissState {
    etag,
    diagnostics_len: context.diagnostics.len(),
  }))
}

fn input_etag(context: &LoaderContext<RunnerContext>) -> Option<Etag> {
  let loader = context.current_loader();
  Some(loader_cache_etag(
    context.content()?,
    loader.options_cache_key(),
    loader.loader_version(),
  ))
}

pub(crate) fn before_normal_loader(
  context: &mut LoaderContext<RunnerContext>,
) -> Result<LoaderCacheAction> {
  debug_assert!(context.current_loader().cache());
  if !context.cacheable {
    return Ok(LoaderCacheAction::Disabled);
  }
  // The minimal cache only supports loaders whose observable input is content and source map.
  if context.additional_data().is_some()
    || !context.parse_meta.is_empty()
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
  {
    return Ok(LoaderCacheAction::Disabled);
  }
  let Some(etag) = input_etag(context) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  let loader_name = context.current_loader().loader_name();
  let module_identifier = context.context.module.identifier();
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    module_identifier.as_str(),
    loader_name,
    etag.clone(),
  );

  if let Some(entry) = item_cache.get::<LoaderCacheEntry>()?
    && loader_cache_dependency_snapshot_is_valid(
      context.context.fs.as_ref(),
      &entry.dependency_snapshot,
    )
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
    let mut dependency_context = LoaderDependencyContext::default();
    restore_loader_cache_dependencies(&entry.dependency_snapshot, &mut dependency_context);
    context.add_dependency_context(&dependency_context);
    context.__finish_with((content, source_map, None));
    return Ok(LoaderCacheAction::Hit);
  }

  Ok(cache_miss_action(context, etag))
}

pub(crate) fn after_normal_loader(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) -> Result<()> {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.additional_data().is_some()
    || !context.parse_meta.is_empty()
  {
    return Ok(());
  }

  if !context.removed_dependency_context().is_empty() {
    return Ok(());
  }
  let Some(dependency_snapshot) = loader_cache_dependency_snapshot(
    context.context.fs.as_ref(),
    context.added_dependency_context(),
  ) else {
    return Ok(());
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
  };
  let loader_name = context.current_loader().loader_name();
  let module_identifier = context.context.module.identifier();
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    module_identifier.as_str(),
    loader_name,
    state.etag.clone(),
  );
  item_cache.store(CacheValue::new(entry))
}
