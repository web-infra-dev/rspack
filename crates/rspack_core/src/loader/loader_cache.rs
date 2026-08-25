use rspack_cacheable::cacheable;
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{Content, LoaderContext, LoaderDependencyContext};
use rspack_paths::{InternedPathSet, Utf8Path};
use rspack_sources::SourceMap;

use crate::{CacheFacade, CacheValue, Etag, ItemCacheFacade, Module, RunnerContext};

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
pub fn loader_cache_dependencies_etag(
  fs: &dyn ReadableFileSystem,
  dependency_context: &LoaderDependencyContext,
) -> Option<Etag> {
  let mut dependencies = dependency_context
    .file_dependencies
    .iter()
    .map(|path| ("file", path))
    .chain(
      dependency_context
        .context_dependencies
        .iter()
        .map(|path| ("context", path)),
    )
    .chain(
      dependency_context
        .build_dependencies
        .iter()
        .map(|path| ("build", path)),
    )
    .collect::<Vec<_>>();
  dependencies.sort_unstable_by(|(kind_a, path_a), (kind_b, path_b)| {
    kind_a
      .cmp(kind_b)
      .then_with(|| path_a.as_path().cmp(path_b.as_path()))
  });

  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  for (kind, path) in dependencies {
    let path = Utf8Path::from_path(path.as_path())?;
    let metadata = fs.metadata_sync(path).ok()?;
    hasher.write(kind.as_bytes());
    hasher.write(path.as_str().as_bytes());
    hasher.write(&metadata.mtime_ms.to_le_bytes());
  }
  Some(Etag::from(format!("{:016x}", hasher.finish())))
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
  dependency_context: LoaderDependencyContext,
  dependencies_etag: Etag,
}

pub(crate) struct LoaderCacheMissState {
  etag: Etag,
  diagnostics_len: usize,
  dependency_context: LoaderDependencyContext,
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
    dependency_context: context.dependency_context.clone(),
  }))
}

fn dependency_delta(
  current: &InternedPathSet,
  previous: &InternedPathSet,
) -> Option<InternedPathSet> {
  if !previous.is_subset(current) {
    return None;
  }
  Some(current.difference(previous).cloned().collect())
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
    && loader_cache_dependencies_etag(context.context.fs.as_ref(), &entry.dependency_context)
      == Some(entry.dependencies_etag.clone())
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
      .dependency_context
      .file_dependencies
      .extend(entry.dependency_context.file_dependencies.iter().cloned());
    context.dependency_context.context_dependencies.extend(
      entry
        .dependency_context
        .context_dependencies
        .iter()
        .cloned(),
    );
    context
      .dependency_context
      .build_dependencies
      .extend(entry.dependency_context.build_dependencies.iter().cloned());
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

  let Some(file_dependencies) = dependency_delta(
    &context.dependency_context.file_dependencies,
    &state.dependency_context.file_dependencies,
  ) else {
    return Ok(());
  };
  let Some(context_dependencies) = dependency_delta(
    &context.dependency_context.context_dependencies,
    &state.dependency_context.context_dependencies,
  ) else {
    return Ok(());
  };
  let Some(build_dependencies) = dependency_delta(
    &context.dependency_context.build_dependencies,
    &state.dependency_context.build_dependencies,
  ) else {
    return Ok(());
  };
  if context.dependency_context.missing_dependencies
    != state.dependency_context.missing_dependencies
  {
    return Ok(());
  }
  let dependency_context = LoaderDependencyContext {
    file_dependencies,
    context_dependencies,
    missing_dependencies: Default::default(),
    build_dependencies,
  };
  let Some(dependencies_etag) =
    loader_cache_dependencies_etag(context.context.fs.as_ref(), &dependency_context)
  else {
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
    dependency_context,
    dependencies_etag,
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
