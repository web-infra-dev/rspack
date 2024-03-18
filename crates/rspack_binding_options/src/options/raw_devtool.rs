use napi::bindgen_prelude::{Either3, Null};
use napi::{Either, Env, JsFunction};
use napi_derive::napi;
use rspack_core::PathData;
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::{get_napi_env, NapiResultExt};
use rspack_plugin_devtool::{
  Append, EvalDevToolModulePluginOptions, ModuleFilenameTemplate, ModuleFilenameTemplateFnCtx,
  SourceMapDevToolPluginOptions, TestFn,
};
use serde::Deserialize;

type RawAppend = Either3<String, bool, JsFunction>;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawPathData {
  pub filename: Option<String>,
  pub content_hash: Option<String>,
  pub url: Option<String>,
}

impl From<PathData<'_>> for RawPathData {
  fn from(ctx: PathData) -> Self {
    RawPathData {
      filename: ctx.filename.map(|s| s.to_string()),
      content_hash: ctx.content_hash.map(|s| s.to_string()),
      url: ctx.url.map(|s| s.to_string()),
    }
  }
}

fn normalize_raw_append(raw: RawAppend) -> Append {
  match raw {
    Either3::A(str) => Append::String(str),
    Either3::B(_) => Append::Disabled,
    Either3::C(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<RawPathData, String>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");
      Append::Fn(Box::new(move |ctx| {
        let fn_payload = fn_payload.clone();
        let value = ctx.into();
        Box::pin(async move {
          fn_payload
            .call(value, ThreadsafeFunctionCallMode::NonBlocking)
            .into_rspack_result()?
            .await
            .unwrap_or_else(|err| panic!("Failed to call append function: {err}"))
        })
      }))
    }
  }
}

type RawFilename = Either3<Null, bool, String>;

type RawModuleFilenameTemplate = Either<String, JsFunction>;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawModuleFilenameTemplateFnCtx {
  pub identifier: String,
  pub short_identifier: String,
  pub resource: String,
  pub resource_path: String,
  pub absolute_resource_path: String,
  pub loaders: String,
  pub all_loaders: String,
  pub query: String,
  pub module_id: String,
  pub hash: String,
  pub namespace: String,
}

impl From<ModuleFilenameTemplateFnCtx> for RawModuleFilenameTemplateFnCtx {
  fn from(ctx: ModuleFilenameTemplateFnCtx) -> Self {
    RawModuleFilenameTemplateFnCtx {
      identifier: ctx.identifier,
      short_identifier: ctx.short_identifier,
      resource: ctx.resource,
      resource_path: ctx.resource_path,
      absolute_resource_path: ctx.absolute_resource_path,
      loaders: ctx.loaders,
      all_loaders: ctx.all_loaders,
      query: ctx.query,
      module_id: ctx.module_id,
      hash: ctx.hash,
      namespace: ctx.namespace,
    }
  }
}

fn normalize_raw_module_filename_template(
  raw: RawModuleFilenameTemplate,
) -> ModuleFilenameTemplate {
  match raw {
    Either::A(str) => ModuleFilenameTemplate::String(str),
    Either::B(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<RawModuleFilenameTemplateFnCtx, String>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");
      ModuleFilenameTemplate::Fn(Box::new(move |ctx| {
        let fn_payload = fn_payload.clone();
        Box::pin(async move {
          fn_payload
            .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
            .into_rspack_result()?
            .await
            .unwrap_or_else(|err| panic!("Failed to call moduleFilenameTemplate function: {err}"))
        })
      }))
    }
  }
}

fn normalize_raw_test(raw: JsFunction) -> TestFn {
  let fn_payload: napi::Result<ThreadsafeFunction<String, bool>> = try {
    let env = get_napi_env();
    rspack_binding_macros::js_fn_into_threadsafe_fn!(raw, &Env::from(env))
  };
  let fn_payload = fn_payload.expect("convert to threadsafe function failed");
  Box::new(move |ctx| {
    fn_payload
      .call(ctx, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()
      .expect("into rspack result failed")
      .blocking_recv()
      .unwrap_or_else(|err| panic!("failed to call external function: {err}"))
      .expect("failed")
  })
}

#[derive(Deserialize)]
#[napi(object)]
pub struct RawSourceMapDevToolPluginOptions {
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(false | null) | string | Function")]
  pub append: Option<RawAppend>,
  pub columns: Option<bool>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | ((info: RawModuleFilenameTemplateFnCtx) => string)")]
  pub fallback_module_filename_template: Option<RawModuleFilenameTemplate>,
  pub file_context: Option<String>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(false | null) | string")]
  pub filename: Option<RawFilename>,
  pub module: Option<bool>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | ((info: RawModuleFilenameTemplateFnCtx) => string)")]
  pub module_filename_template: Option<RawModuleFilenameTemplate>,
  pub namespace: Option<String>,
  pub no_sources: Option<bool>,
  pub public_path: Option<String>,
  pub source_root: Option<String>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(text: string) => boolean")]
  pub test: Option<JsFunction>,
}

impl From<RawSourceMapDevToolPluginOptions> for SourceMapDevToolPluginOptions {
  fn from(opts: RawSourceMapDevToolPluginOptions) -> Self {
    let append = opts.append.map(normalize_raw_append);
    let test = opts.test.map(normalize_raw_test);
    let filename = opts.filename.and_then(|raw| match raw {
      Either3::A(_) | Either3::B(_) => None,
      Either3::C(s) => Some(s),
    });

    let module_filename_template = opts
      .module_filename_template
      .map(normalize_raw_module_filename_template);
    let fallback_module_filename_template = opts
      .fallback_module_filename_template
      .map(normalize_raw_module_filename_template);

    let columns = opts.columns.unwrap_or(true);
    let no_sources = opts.no_sources.unwrap_or(false);

    Self {
      append,
      columns,
      fallback_module_filename_template,
      file_context: opts.file_context,
      filename,
      namespace: opts.namespace,
      no_sources,
      public_path: opts.public_path,
      module_filename_template,
      module: opts.module.unwrap_or(false),
      source_root: opts.source_root,
      test,
    }
  }
}

#[derive(Deserialize)]
#[napi(object)]
pub struct RawEvalDevToolModulePluginOptions {
  pub namespace: Option<String>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | ((info: RawModuleFilenameTemplateFnCtx) => string)")]
  pub module_filename_template: Option<RawModuleFilenameTemplate>,
  pub source_url_comment: Option<String>,
}

impl From<RawEvalDevToolModulePluginOptions> for EvalDevToolModulePluginOptions {
  fn from(opts: RawEvalDevToolModulePluginOptions) -> Self {
    let module_filename_template = opts
      .module_filename_template
      .map(normalize_raw_module_filename_template);

    Self {
      namespace: opts.namespace,
      source_url_comment: opts.source_url_comment,
      module_filename_template,
    }
  }
}
