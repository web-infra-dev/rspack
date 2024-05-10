use std::sync::Arc;

use napi::bindgen_prelude::{Either3, Null};
use napi::Either;
use napi_derive::napi;
use rspack_core::PathData;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_devtool::{
  Append, EvalDevToolModulePluginOptions, ModuleFilenameTemplate, ModuleFilenameTemplateFnCtx,
  SourceMapDevToolPluginOptions, TestFn,
};
use tokio::runtime::Handle;

type RawAppend = Either3<String, bool, ThreadsafeFunction<RawPathData, String>>;

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
    Either3::C(v) => Append::Fn(Box::new(move |ctx| {
      let v = v.clone();
      let value = ctx.into();
      Box::pin(async move { v.call(value).await })
    })),
  }
}

type RawFilename = Either3<Null, bool, String>;

type RawModuleFilenameTemplate =
  Either<String, ThreadsafeFunction<RawModuleFilenameTemplateFnCtx, String>>;

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
    Either::B(v) => ModuleFilenameTemplate::Fn(Arc::new(move |ctx| {
      let v = v.clone();
      Box::pin(async move { v.call(ctx.into()).await })
    })),
  }
}

fn normalize_raw_test(raw: ThreadsafeFunction<String, bool>) -> TestFn {
  let handle = Handle::current();
  Box::new(move |ctx| handle.block_on(raw.call(ctx)))
}

#[napi(object, object_to_js = false)]
pub struct RawSourceMapDevToolPluginOptions {
  #[napi(ts_type = "(false | null) | string | Function")]
  pub append: Option<RawAppend>,
  pub columns: Option<bool>,
  #[napi(ts_type = "string | ((info: RawModuleFilenameTemplateFnCtx) => string)")]
  pub fallback_module_filename_template: Option<RawModuleFilenameTemplate>,
  pub file_context: Option<String>,
  #[napi(ts_type = "(false | null) | string")]
  pub filename: Option<RawFilename>,
  pub module: Option<bool>,
  #[napi(ts_type = "string | ((info: RawModuleFilenameTemplateFnCtx) => string)")]
  pub module_filename_template: Option<RawModuleFilenameTemplate>,
  pub namespace: Option<String>,
  pub no_sources: Option<bool>,
  pub public_path: Option<String>,
  pub source_root: Option<String>,
  #[napi(ts_type = "(text: string) => boolean")]
  pub test: Option<ThreadsafeFunction<String, bool>>,
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

#[napi(object, object_to_js = false)]
pub struct RawEvalDevToolModulePluginOptions {
  pub namespace: Option<String>,
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
