use rspack_cacheable::cacheable;
use rspack_collections::Identifiable;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{Content, LoaderContext};
use rspack_sources::SourceMap;

use crate::{CacheFacade, CacheValue, Etag, ItemCacheFacade, Module, RunnerContext};

#[tracing::instrument(
  "loader_cache:key",
  skip_all,
  level = "trace",
  target = "rspack_loader_cache",
  fields(
    perfetto.track_name = "loader_cache:key",
    perfetto.process_name = "Loader Analysis",
  )
)]
fn loader_cache_key(module_identifier: &str, loader_name: &str) -> String {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "module" => module_identifier,
    "loader" => loader_name,
  });
  format!("{:016x}", hasher.finish())
}

#[doc(hidden)]
#[tracing::instrument(
  "loader_cache:etag",
  skip_all,
  level = "trace",
  target = "rspack_loader_cache",
  fields(
    perfetto.track_name = "loader_cache:etag",
    perfetto.process_name = "Loader Analysis",
  )
)]
pub fn loader_cache_etag(
  content: &Content,
  source_map: Option<&[u8]>,
  options_cache_key: &str,
  loader_version: &str,
) -> Etag {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  rspack_hash::rspack_hash_object!(&mut hasher, {
    "content" => content,
    "sourceMap" => source_map,
    "options" => options_cache_key,
    "version" => loader_version,
  });
  Etag::from(format!("{:016x}", hasher.finish()))
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
  parse_meta: String,
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
  let source_map = {
    let span = tracing::trace_span!(
      target: "rspack_loader_cache",
      "loader_cache:input_source_map",
      perfetto.track_name = "loader_cache:input_source_map",
      perfetto.process_name = "Loader Analysis",
    );
    let _entered = span.enter();
    context.source_map().map(SourceMap::to_json)
  };
  Some(loader_cache_etag(
    context.content()?,
    source_map.as_deref().map(str::as_bytes),
    loader.options_cache_key(),
    loader.loader_version(),
  ))
}

#[tracing::instrument(
  "loader_cache:before",
  skip_all,
  level = "trace",
  target = "rspack_loader_cache",
  fields(
    perfetto.track_name = "loader_cache:before",
    perfetto.process_name = "Loader Analysis",
    loader = context.current_loader().loader_name(),
    resource = context.resource(),
  )
)]
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

  let entry = {
    let span = tracing::trace_span!(
      target: "rspack_loader_cache",
      "loader_cache:get",
      perfetto.track_name = "loader_cache:get",
      perfetto.process_name = "Loader Analysis",
    );
    let _entered = span.enter();
    item_cache.get::<LoaderCacheEntry>()?
  };

  if let Some(entry) = entry {
    let span = tracing::trace_span!(
      target: "rspack_loader_cache",
      "loader_cache:restore",
      perfetto.track_name = "loader_cache:restore",
      perfetto.process_name = "Loader Analysis",
    );
    let _entered = span.enter();
    let content = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:content_deserialize",
        perfetto.track_name = "loader_cache:content_deserialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      match (&entry.content, entry.content_is_string) {
        (Some(content), true) => {
          // SAFETY: String cache entries are written exclusively from `Content::String`.
          let content = unsafe { String::from_utf8_unchecked(content.clone()) };
          Some(Content::String(content))
        }
        (Some(content), false) => Some(Content::Buffer(content.clone())),
        (None, _) => None,
      }
    };
    let source_map = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:source_map_deserialize",
        perfetto.track_name = "loader_cache:source_map_deserialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      entry
        .source_map
        .clone()
        .and_then(|source_map| SourceMap::from_json(source_map).ok())
    };
    context.parse_meta = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:parse_meta_deserialize",
        perfetto.track_name = "loader_cache:parse_meta_deserialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      serde_json::from_str(&entry.parse_meta).to_rspack_result()?
    };
    context.__finish_with((content, source_map, None));
    return Ok(LoaderCacheAction::Hit);
  }

  Ok(cache_miss_action(context, etag))
}

#[tracing::instrument(
  "loader_cache:after",
  skip_all,
  level = "trace",
  target = "rspack_loader_cache",
  fields(
    perfetto.track_name = "loader_cache:after",
    perfetto.process_name = "Loader Analysis",
    loader = context.current_loader().loader_name(),
    resource = context.resource(),
  )
)]
pub(crate) fn after_normal_loader(
  context: &LoaderContext<RunnerContext>,
  state: &LoaderCacheMissState,
) -> Result<()> {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || !context.context.module.build_info().extras.is_empty()
    || context.additional_data().is_some()
  {
    return Ok(());
  }

  // Dynamic loader dependencies are intentionally outside the minimal cache
  // contract and are neither stored nor replayed on a cache hit.
  let entry = {
    let span = tracing::trace_span!(
      target: "rspack_loader_cache",
      "loader_cache:serialize",
      perfetto.track_name = "loader_cache:serialize",
      perfetto.process_name = "Loader Analysis",
    );
    let _entered = span.enter();
    let (content, content_is_string) = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:content_serialize",
        perfetto.track_name = "loader_cache:content_serialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      match context.content() {
        Some(Content::String(content)) => (Some(content.as_bytes().to_vec()), true),
        Some(Content::Buffer(content)) => (Some(content.clone()), false),
        None => (None, false),
      }
    };
    let source_map = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:source_map_serialize",
        perfetto.track_name = "loader_cache:source_map_serialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      context.source_map().map(SourceMap::to_json)
    };
    let parse_meta = {
      let span = tracing::trace_span!(
        target: "rspack_loader_cache",
        "loader_cache:parse_meta_serialize",
        perfetto.track_name = "loader_cache:parse_meta_serialize",
        perfetto.process_name = "Loader Analysis",
      );
      let _entered = span.enter();
      serde_json::to_string(&context.parse_meta).to_rspack_result()?
    };
    LoaderCacheEntry {
      content,
      content_is_string,
      source_map,
      parse_meta,
    }
  };
  let loader_name = context.current_loader().loader_name();
  let module_identifier = context.context.module.identifier();
  let item_cache = loader_cache_item(
    &context.context.loader_cache,
    module_identifier.as_str(),
    loader_name,
    state.etag.clone(),
  );
  let span = tracing::trace_span!(
    target: "rspack_loader_cache",
    "loader_cache:store",
    perfetto.track_name = "loader_cache:store",
    perfetto.process_name = "Loader Analysis",
  );
  let _entered = span.enter();
  item_cache.store(CacheValue::new(entry))
}
