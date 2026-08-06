use std::{
  ptr::NonNull,
  sync::{
    Arc, LazyLock, RwLock,
    atomic::{AtomicU32, Ordering},
  },
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{
  AdditionalData, LoaderCacheContext, LoaderCacheEntry, LoaderContext, Module, RunnerContext,
};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
use rustc_hash::FxHashMap as HashMap;

use crate::{error::RspackError, module::ModuleObject};

#[napi(object)]
#[derive(Hash)]
pub struct JsLoaderItem {
  pub loader: String,
  pub r#type: String,

  // data
  pub data: serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,

  pub no_pitch: bool,
  pub cache: bool,
}

impl From<&rspack_loader_runner::LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: &rspack_loader_runner::LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      loader: value.request().to_string(),
      r#type: value.r#type().to_string(),

      data: value.data().clone(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),

      no_pitch: false,
      cache: value.cache(),
    }
  }
}

impl<C> From<&Arc<dyn rspack_core::Loader<C>>> for JsLoaderItem
where
  C: Send,
{
  fn from(loader: &Arc<dyn rspack_core::Loader<C>>) -> Self {
    let identifier = loader.identifier();

    if let Some((r#type, ident)) = identifier.split_once('|') {
      return Self {
        loader: ident.to_string(),
        data: serde_json::Value::Null,
        r#type: r#type.to_string(),
        pitch_executed: false,
        normal_executed: false,
        no_pitch: false,
        cache: false,
      };
    }
    Self {
      loader: identifier.to_string(),
      data: serde_json::Value::Null,
      r#type: String::default(),
      pitch_executed: false,
      normal_executed: false,
      no_pitch: false,
      cache: false,
    }
  }
}

#[napi(string_enum)]
pub enum JsLoaderState {
  Pitching,
  Normal,
}

impl From<LoaderState> for JsLoaderState {
  fn from(value: LoaderState) -> Self {
    match value {
      LoaderState::Init | LoaderState::ProcessResource | LoaderState::Finished => {
        panic!("Unexpected loader runner state: {value:?}")
      }
      LoaderState::Pitching => JsLoaderState::Pitching,
      LoaderState::Normal => JsLoaderState::Normal,
    }
  }
}

#[napi(object)]
pub struct JsLoaderCacheEntry {
  pub content: Buffer,
  pub source_map: Option<Buffer>,
  pub utf8_hint: bool,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
}

impl From<Arc<LoaderCacheEntry>> for JsLoaderCacheEntry {
  fn from(entry: Arc<LoaderCacheEntry>) -> Self {
    Self {
      utf8_hint: matches!(&entry.content, rspack_core::Content::String(_)),
      content: entry.content.clone().into_bytes().into(),
      source_map: entry
        .source_map
        .as_ref()
        .map(|source_map| source_map.clone().into_bytes().into()),
      additional_data: entry
        .additional_data
        .as_ref()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      file_dependencies: entry
        .file_dependencies
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      context_dependencies: entry
        .context_dependencies
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      missing_dependencies: entry
        .missing_dependencies
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      build_dependencies: entry
        .build_dependencies
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
    }
  }
}

static NEXT_LOADER_CACHE_ID: AtomicU32 = AtomicU32::new(1);
static LOADER_CACHE_CONTEXTS: LazyLock<RwLock<HashMap<u32, LoaderCacheContext>>> =
  LazyLock::new(Default::default);

fn register_loader_cache(cache: LoaderCacheContext) -> u32 {
  let id = NEXT_LOADER_CACHE_ID.fetch_add(1, Ordering::Relaxed);
  LOADER_CACHE_CONTEXTS
    .write()
    .expect("loader cache contexts should not be poisoned")
    .insert(id, cache);
  id
}

pub(crate) struct LoaderCacheGuard(u32);

impl LoaderCacheGuard {
  pub(crate) fn new(id: u32) -> Self {
    Self(id)
  }
}

impl Drop for LoaderCacheGuard {
  fn drop(&mut self) {
    if self.0 != 0 {
      LOADER_CACHE_CONTEXTS
        .write()
        .expect("loader cache contexts should not be poisoned")
        .remove(&self.0);
    }
  }
}

#[napi(js_name = "__internal__getLoaderCache")]
pub fn get_loader_cache(cache_id: u32, loader_index: i32) -> Option<JsLoaderCacheEntry> {
  LOADER_CACHE_CONTEXTS
    .read()
    .expect("loader cache contexts should not be poisoned")
    .get(&cache_id)
    .and_then(|cache| cache.get(loader_index))
    .map(Into::into)
}

#[napi(js_name = "__internal__setLoaderCache")]
pub fn set_loader_cache(
  cache_id: u32,
  loader_index: i32,
  mut entry: JsLoaderCacheEntry,
) -> napi::Result<()> {
  let source_map = entry
    .source_map
    .map(Into::<Vec<u8>>::into)
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| napi::Error::from_reason(error.to_string()))?;
  let content = if entry.utf8_hint {
    String::from_utf8(Into::<Vec<u8>>::into(entry.content))
      .map(Into::into)
      .map_err(|error| napi::Error::from_reason(error.to_string()))?
  } else {
    Into::<Vec<u8>>::into(entry.content).into()
  };
  let additional_data = entry.additional_data.take().map(|data| {
    let mut additional_data = AdditionalData::default();
    additional_data.insert(data);
    additional_data
  });
  let cache = LOADER_CACHE_CONTEXTS
    .read()
    .expect("loader cache contexts should not be poisoned")
    .get(&cache_id)
    .cloned();
  if let Some(cache) = cache {
    cache.insert(
      loader_index,
      LoaderCacheEntry {
        content,
        source_map,
        additional_data,
        file_dependencies: entry
          .file_dependencies
          .into_iter()
          .map(Into::into)
          .collect(),
        context_dependencies: entry
          .context_dependencies
          .into_iter()
          .map(Into::into)
          .collect(),
        missing_dependencies: entry
          .missing_dependencies
          .into_iter()
          .map(Into::into)
          .collect(),
        build_dependencies: entry
          .build_dependencies
          .into_iter()
          .map(Into::into)
          .collect(),
      },
    );
  }
  Ok(())
}

#[napi(object)]
pub struct JsLoaderContext {
  pub resource: String,
  #[napi(js_name = "_module", ts_type = "Module")]
  pub module: ModuleObject,
  #[napi(ts_type = "Readonly<boolean>")]
  pub hot: bool,

  /// Content maybe empty in pitching stage
  pub content: Either<Null, Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  #[napi(js_name = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, String>,
  pub source_map: Option<Buffer>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,

  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
  #[napi(js_name = "__internal__error")]
  pub error: Option<RspackError>,

  /// UTF-8 hint for `content`
  /// - Some(true): `content` is a `UTF-8` encoded sequence
  #[napi(js_name = "__internal__utf8Hint")]
  pub utf8_hint: Option<bool>,
  #[napi(js_name = "__internal__loaderCache")]
  pub loader_cache: u32,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = &cx.context.module;

    #[allow(clippy::unwrap_used)]
    Ok(JsLoaderContext {
      resource: cx.resource_data.resource().to_owned(),
      module: ModuleObject::with_ptr(
        NonNull::new(module.as_ref() as *const dyn Module as *mut dyn Module).unwrap(),
        cx.context.compiler_id,
      ),
      hot: cx.hot,
      content: match cx.content() {
        Some(c) => Either::B(c.to_owned().into_bytes().into()),
        None => Either::A(Null),
      },
      // Since js side only set parse meta, and can't read it, so we can use Default here to only bring the
      // set values from js side to rust side.
      parse_meta: Default::default(),
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx
        .source_map()
        .map(|v| v.to_json())
        .map(|v| v.into_bytes().into()),
      cacheable: cx.cacheable,
      file_dependencies: cx
        .file_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      context_dependencies: cx
        .context_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      missing_dependencies: cx
        .missing_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      build_dependencies: cx
        .build_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),

      loader_items: cx.loader_items.iter().map(Into::into).collect(),
      loader_index: cx.loader_index,
      loader_state: cx.state().into(),
      error: None,
      utf8_hint: None,
      loader_cache: if cx.loader_items.iter().any(|loader| loader.cache()) {
        register_loader_cache(cx.context.loader_cache.clone())
      } else {
        0
      },
    })
  }
}
