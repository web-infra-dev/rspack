use std::sync::Arc;

use derivative::Derivative;
use napi::bindgen_prelude::Either3;
use napi::{Either, Env, JsFunction};
use napi_derive::napi;
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::{get_napi_env, NapiResultExt};
use rspack_plugin_devtool::{
  Append, ModuleFilenameTemplate, ModuleFilenameTemplateFnCtx, SourceMapDevToolPluginOptions,
};
use serde::Deserialize;

type RawAppend = Either3<String, bool, JsFunction>;

fn normalize_raw_append(raw: RawAppend) -> Append {
  match raw {
    Either3::A(str) => Append::String(str),
    Either3::B(_) => Append::Disabled,
    Either3::C(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<(), Option<String>>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");
      Append::Fn(Arc::new(move || {
        fn_payload
          .call((), ThreadsafeFunctionCallMode::NonBlocking)
          .into_rspack_result()
          .expect("into rspack result failed")
          .blocking_recv()
          .unwrap_or_else(|err| panic!("Failed to call external function: {err}"))
          .expect("failed")
      }))
    }
  }
}

type RawFilename = Either<bool, String>;

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
) -> Option<ModuleFilenameTemplate> {
  match raw {
    Either::A(str) => Some(ModuleFilenameTemplate::String(str)),
    Either::B(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<RawModuleFilenameTemplateFnCtx, String>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");

      Some(ModuleFilenameTemplate::Fn(Arc::new(move |ctx| {
        fn_payload
          .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
          .into_rspack_result()
          .expect("into rspack result failed")
          .blocking_recv()
          .unwrap_or_else(|err| panic!("Failed to call external function: {err}"))
          .expect("failed")
      })))
    }
  }
}

#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
#[napi(object)]
pub struct RawSourceMapDevToolPluginOptions {
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(false | null) | string | Function")]
  #[derivative(Debug = "ignore")]
  pub append: Option<RawAppend>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(false | null) | string")]
  pub filename: Option<RawFilename>,
  pub namespace: Option<String>,
  pub columns: Option<bool>,
  pub no_sources: Option<bool>,
  pub public_path: Option<String>,
  pub module: Option<bool>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | Function")]
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<RawModuleFilenameTemplate>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | Function")]
  #[derivative(Debug = "ignore")]
  pub fallback_module_filename_template: Option<RawModuleFilenameTemplate>,
}

impl From<RawSourceMapDevToolPluginOptions> for SourceMapDevToolPluginOptions {
  fn from(opts: RawSourceMapDevToolPluginOptions) -> Self {
    let append = opts.append.map(|name| normalize_raw_append(name));

    let filename = match opts.filename {
      Some(raw) => match raw {
        Either::A(_) => None,
        Either::B(s) => Some(s),
      },
      None => None,
    };

    let module_filename_template = match opts.module_filename_template {
      Some(value) => normalize_raw_module_filename_template(value),
      None => None,
    };

    let fallback_module_filename_template = match opts.fallback_module_filename_template {
      Some(value) => normalize_raw_module_filename_template(value),
      None => None,
    };

    let columns = match opts.columns {
      Some(b) => b,
      None => true,
    };

    let no_sources = match opts.no_sources {
      Some(b) => b,
      None => false,
    };

    Self {
      append,
      filename,
      namespace: opts.namespace,
      columns,
      no_sources,
      public_path: opts.public_path,
      module_filename_template,
      fallback_module_filename_template,
      module: opts.module.unwrap_or(false),
    }
  }
}
