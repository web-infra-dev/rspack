use std::{ops::Deref, path::PathBuf, str::FromStr};

use napi_derive::napi;
use rspack_core::{rspack_sources::SourceMap, Content, ResourceData};
use rspack_error::Diagnostic;
use rspack_loader_runner::AdditionalData;
use rustc_hash::FxHashSet as HashSet;
use {
  napi::bindgen_prelude::*,
  rspack_core::{Loader, LoaderContext, LoaderRunnerContext},
  rspack_error::error,
  rspack_identifier::{Identifiable, Identifier},
  rspack_napi::threadsafe_function::ThreadsafeFunction,
  rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef,
};

use crate::get_builtin_loader;

type ThreadsafeLoaderRunner =
  ThreadsafeFunction<JsLoaderContext, Promise<LoaderThreadsafeLoaderResult>>;

/// Loader Runner for JavaScript environment
#[derive(Clone)]
pub struct JsLoaderRunner(ThreadsafeLoaderRunner);

impl Deref for JsLoaderRunner {
  type Target = ThreadsafeLoaderRunner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromNapiValue for JsLoaderRunner {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    unsafe { <JsFunction as ValidateNapiValue>::validate(env, napi_val) }?;
    Ok(Self(unsafe {
      FromNapiValue::from_napi_value(env, napi_val)
    }?))
  }
}

pub struct JsLoaderAdapter {
  pub runner: JsLoaderRunner,
  pub identifier: Identifier,
}

impl std::fmt::Debug for JsLoaderAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsLoaderAdapter")
      .field("loaders", &self.identifier)
      .finish_non_exhaustive()
  }
}

impl Identifiable for JsLoaderAdapter {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for JsLoaderAdapter {
  async fn pitch(
    &self,
    loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
  ) -> rspack_error::Result<()> {
    let mut js_loader_context: JsLoaderContext = loader_context.try_into()?;
    js_loader_context.is_pitching = true;

    let loader_result = self.runner.call_with_promise(js_loader_context).await?;

    if let Some(loader_result) = loader_result {
      // This indicate that the JS loaders pitched(return something) successfully
      // and executed the normal loader on the JS loader side(in that group),
      // then here we want to change the control flow in order
      // to execute the remaining normal loaders on the native side.
      if !loader_result.is_pitching {
        loader_context
          .current_loader()
          .__do_not_use_or_you_will_be_fired_set_normal_executed();
      }
      sync_loader_context(loader_result, loader_context)?;
    }

    Ok(())
  }
  async fn run(
    &self,
    loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
  ) -> rspack_error::Result<()> {
    let mut js_loader_context: JsLoaderContext = loader_context.try_into()?;
    // Instruct the JS loader-runner to execute loaders in backwards.
    js_loader_context.is_pitching = false;

    let loader_result = self.runner.call_with_promise(js_loader_context).await?;

    if let Some(loader_result) = loader_result {
      sync_loader_context(loader_result, loader_context)?;
    }

    Ok(())
  }
}

fn sync_loader_context(
  loader_result: JsLoaderResult,
  loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
) -> rspack_error::Result<()> {
  loader_context.cacheable = loader_result.cacheable;
  loader_context.file_dependencies = loader_result
    .file_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.context_dependencies = loader_result
    .context_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.missing_dependencies = loader_result
    .missing_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.build_dependencies = loader_result
    .build_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  loader_context.content = loader_result
    .content
    .map(|c| rspack_core::Content::from(Into::<Vec<u8>>::into(c)));
  loader_context.source_map = loader_result
    .source_map
    .as_ref()
    .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
    .transpose()
    .map_err(|e| error!(e.to_string()))?;
  loader_context.additional_data = loader_result.additional_data_external.clone();
  if let Some(data) = loader_result.additional_data {
    loader_context.additional_data.insert(data);
  } else {
    loader_context
      .additional_data
      .remove::<ThreadsafeJsValueRef<Unknown>>();
  }
  loader_context.asset_filenames = loader_result.asset_filenames.into_iter().collect();

  Ok(())
}

#[napi(object)]
pub struct JsLoaderContext {
  /// Content maybe empty in pitching stage
  pub content: Either<Null, Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  pub source_map: Option<Buffer>,
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub asset_filenames: Vec<String>,

  pub current_loader: String,
  pub is_pitching: bool,
  /// Loader index from JS.
  /// If loaders are dispatched by JS loader runner,
  /// then, this field is correspondence with loader index in JS side.
  /// It is useful when loader dispatched on JS side has an builtin loader, for example: builtin:swc-loader,
  /// Then this field will be used as an hack to test whether it should return an AST or string.
  pub loader_index_from_js: Option<u32>,
  /// Internal additional data, contains more than `String`
  /// @internal
  #[napi(ts_type = "ExternalObject<'AdditionalData'>")]
  pub additional_data_external: External<AdditionalData>,
  /// Internal loader context
  /// @internal
  #[napi(ts_type = "ExternalObject<'LoaderRunnerContext'>")]
  pub context_external: External<rspack_core::LoaderRunnerContext>,
  /// Internal loader diagnostic
  /// @internal
  #[napi(ts_type = "ExternalObject<'Diagnostic[]'>")]
  pub diagnostics_external: External<Vec<Diagnostic>>,

  #[napi(js_name = "_moduleIdentifier")]
  pub module_identifier: String,

  pub hot: bool,
}

impl TryFrom<&mut rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>>
  for JsLoaderContext
{
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<'_, rspack_core::LoaderRunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    Ok(JsLoaderContext {
      content: match &cx.content {
        Some(c) => Either::B(c.to_owned().into_bytes().into()),
        None => Either::A(Null),
      },
      additional_data: cx
        .additional_data
        .get::<ThreadsafeJsValueRef<Unknown>>()
        .cloned(),
      source_map: cx
        .source_map
        .clone()
        .map(|v| v.to_json())
        .transpose()
        .map_err(|e| error!(e.to_string()))?
        .map(|v| v.into_bytes().into()),
      resource: cx.resource().to_owned(),
      resource_path: cx.resource_path().to_string_lossy().to_string(),
      resource_fragment: cx.resource_fragment().map(|r| r.to_owned()),
      resource_query: cx.resource_query().map(|r| r.to_owned()),
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
      asset_filenames: cx.asset_filenames.iter().map(|i| i.to_owned()).collect(),

      current_loader: cx.current_loader().to_string(),
      is_pitching: true,
      loader_index_from_js: None,

      additional_data_external: External::new(cx.additional_data.clone()),
      context_external: External::new(cx.context.clone()),
      diagnostics_external: External::new(cx.__diagnostics.drain(..).collect()),
      module_identifier: cx.context.module.to_string(),
      hot: cx.hot,
    })
  }
}

pub async fn run_builtin_loader(
  builtin: String,
  options: Option<&str>,
  loader_context: JsLoaderContext,
) -> Result<JsLoaderContext> {
  use rspack_loader_runner::__private::loader::LoaderItemList;

  let loader = get_builtin_loader(&builtin, options);
  let loader_item = loader.clone().into();
  let list = &[loader_item];
  let additional_data = {
    let mut additional_data = loader_context.additional_data_external.clone();
    if let Some(data) = loader_context.additional_data {
      additional_data.insert(data);
    }
    additional_data
  };

  let mut resource_data = ResourceData::new(
    loader_context.resource,
    PathBuf::from(loader_context.resource_path),
  )
  .query_optional(loader_context.resource_query)
  .fragment_optional(loader_context.resource_fragment);

  let mut cx = LoaderContext {
    hot: loader_context.hot,
    content: match loader_context.content {
      Either::A(_) => None,
      Either::B(c) => Some(Content::from(c.as_ref().to_owned())),
    },
    context: loader_context.context_external.clone(),
    source_map: loader_context
      .source_map
      .map(|s| SourceMap::from_slice(s.as_ref()))
      .transpose()
      .map_err(|e| Error::from_reason(e.to_string()))?,
    additional_data,
    cacheable: loader_context.cacheable,
    file_dependencies: HashSet::from_iter(
      loader_context
        .file_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    context_dependencies: HashSet::from_iter(
      loader_context
        .context_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    missing_dependencies: HashSet::from_iter(
      loader_context
        .missing_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    build_dependencies: HashSet::from_iter(
      loader_context
        .build_dependencies
        .iter()
        .map(|m| PathBuf::from_str(m).expect("Should convert to path")),
    ),
    asset_filenames: HashSet::from_iter(loader_context.asset_filenames.into_iter()),
    // Initialize with no diagnostic
    __diagnostics: vec![],
    __resource_data: &mut resource_data,
    __loader_items: LoaderItemList(list),
    // This is used an hack to `builtin:swc-loader` in order to determine whether to return AST or source.
    __loader_index: loader_context.loader_index_from_js.unwrap_or(0) as usize,
    __plugins: &[],
  };
  if loader_context.is_pitching {
    // Builtin loaders dispatched using JS loader-runner does not support pitching.
    // This phase is ignored.
  } else {
    // Run normal loader
    loader
      .run(&mut cx)
      .await
      .map_err(|e| Error::from_reason(e.to_string()))?;
    // restore the hack
    cx.__loader_index = 0;
  }

  JsLoaderContext::try_from(&mut cx).map_err(|e| Error::from_reason(e.to_string()))
}

// #[napi(object)]
pub struct JsLoaderResult {
  /// Content in pitching stage can be empty
  pub content: Option<Buffer>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub asset_filenames: Vec<String>,
  pub source_map: Option<Buffer>,
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  pub additional_data_external: External<AdditionalData>,
  pub cacheable: bool,
  /// Used to instruct how rust loaders should execute
  pub is_pitching: bool,
}

/// Only for dts generation
#[napi(object, object_to_js = false, js_name = "JsLoaderResult")]
pub struct JsLoaderResult_ {
  /// Content in pitching stage can be empty
  pub content: Option<Buffer>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub asset_filenames: Vec<String>,
  pub source_map: Option<Buffer>,
  pub additional_data: Option<Buffer>,
  #[napi(ts_type = "ExternalObject<'AdditionalData'>")]
  pub additional_data_external: External<AdditionalData>,
  pub cacheable: bool,
  /// Used to instruct how rust loaders should execute
  pub is_pitching: bool,
}

impl napi::bindgen_prelude::TypeName for JsLoaderResult {
  fn type_name() -> &'static str {
    "JsLoaderResult"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

// Manually convert
impl napi::bindgen_prelude::FromNapiValue for JsLoaderResult {
  unsafe fn from_napi_value(
    env: napi::bindgen_prelude::sys::napi_env,
    napi_val: napi::bindgen_prelude::sys::napi_value,
  ) -> napi::bindgen_prelude::Result<Self> {
    let obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
    let content_: Option<Buffer> = obj.get("content")?;
    let asset_filenames_: Vec<String> = obj.get("assetFilenames")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "assetFilenames"),
      )
    })?;
    let file_dependencies_: Vec<String> = obj.get("fileDependencies")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "fileDependencies"),
      )
    })?;
    let context_dependencies_: Vec<String> = obj.get("contextDependencies")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "contextDependencies"),
      )
    })?;
    let missing_dependencies_: Vec<String> = obj.get("missingDependencies")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "missingDependencies"),
      )
    })?;
    let build_dependencies_: Vec<String> = obj.get("buildDependencies")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "buildDependencies"),
      )
    })?;
    let source_map_: Option<Buffer> = obj.get("sourceMap")?;
    let additional_data_: Option<ThreadsafeJsValueRef<Unknown>> =
      obj.get::<_, ThreadsafeJsValueRef<Unknown>>("additionalData")?;

    // change: eagerly clone this field since `External<T>` might be dropped.
    let additional_data_external_: External<AdditionalData> = obj
      .get("additionalDataExternal")?
      .map(|v: External<AdditionalData>| External::new(v.clone()))
      .ok_or_else(|| {
        napi::bindgen_prelude::Error::new(
          napi::bindgen_prelude::Status::InvalidArg,
          format!("Missing field `{}`", "additionalDataExternal"),
        )
      })?;
    let cacheable_: bool = obj.get("cacheable")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "cacheable"),
      )
    })?;
    let is_pitching_: bool = obj.get("isPitching")?.ok_or_else(|| {
      napi::bindgen_prelude::Error::new(
        napi::bindgen_prelude::Status::InvalidArg,
        format!("Missing field `{}`", "isPitching"),
      )
    })?;
    let val = Self {
      content: content_,
      file_dependencies: file_dependencies_,
      context_dependencies: context_dependencies_,
      missing_dependencies: missing_dependencies_,
      build_dependencies: build_dependencies_,
      asset_filenames: asset_filenames_,
      source_map: source_map_,
      additional_data: additional_data_,
      additional_data_external: additional_data_external_,
      cacheable: cacheable_,
      is_pitching: is_pitching_,
    };
    Ok(val)
  }
}
impl ValidateNapiValue for JsLoaderResult {}

pub type LoaderThreadsafeLoaderResult = Option<JsLoaderResult>;
