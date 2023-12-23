use std::sync::Arc;

use derivative::Derivative;
use napi::{bindgen_prelude::Either3, Either, Env, JsFunction};
use napi_derive::napi;
use rspack_napi_shared::{
  get_napi_env,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsRegExp, NapiResultExt,
};
use rspack_plugin_devtool::{
  Append, DevtoolPluginOptions, FallbackModuleFilenameTemplate, ModuleFilenameTemplate,
};
use serde::Deserialize;

type RawAppend = Either3<String, bool, JsFunction>;

#[inline]
fn default_append() -> Append {
  Append::Default
}

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

type RawRule = Either<String, JsRegExp>;
type RawRules = Either<RawRule, Vec<RawRule>>;

type RawFallbackModuleFilenameTemplate = Either3<String, bool, JsFunction>;

#[inline]
fn default_fallback_module_filename_template() -> FallbackModuleFilenameTemplate {
  FallbackModuleFilenameTemplate::Disabled
}

fn normalize_raw_fallback_module_filename_template(
  raw: RawFallbackModuleFilenameTemplate,
) -> FallbackModuleFilenameTemplate {
  match raw {
    Either3::A(str) => FallbackModuleFilenameTemplate::String(str),
    Either3::B(_) => FallbackModuleFilenameTemplate::Disabled,
    Either3::C(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<(), Option<String>>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");
      FallbackModuleFilenameTemplate::Fn(Arc::new(move || {
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

type RawModuleFilenameTemplate = Either3<String, bool, JsFunction>;

#[inline]
fn default_module_filename_template() -> ModuleFilenameTemplate {
  ModuleFilenameTemplate::Disabled
}

fn normalize_raw_module_filename_template(
  raw: RawModuleFilenameTemplate,
) -> ModuleFilenameTemplate {
  match raw {
    Either3::A(str) => ModuleFilenameTemplate::String(str),
    Either3::B(_) => ModuleFilenameTemplate::Disabled,
    Either3::C(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<(), Option<String>>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = fn_payload.expect("convert to threadsafe function failed");
      ModuleFilenameTemplate::Fn(Arc::new(move || {
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

#[derive(Derivative, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[derivative(Debug)]
pub struct RawSourceMapDevToolPluginOptions {
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(false | null) | string | Function")]
  #[derivative(Debug = "ignore")]
  pub append: Option<RawAppend>,
  pub columns: Option<bool>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  #[derivative(Debug = "ignore")]
  pub exclude: Option<RawRules>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | false | Function")]
  #[derivative(Debug = "ignore")]
  pub fallback_module_filename_template: Option<RawFallbackModuleFilenameTemplate>,
  pub file_context: Option<String>,
  #[napi(ts_type = "(false | null) | string")]
  pub filename: Option<String>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  #[derivative(Debug = "ignore")]
  pub include: Option<RawRules>,
  pub module: Option<bool>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | false | Function")]
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<RawModuleFilenameTemplate>,
  pub namespace: Option<String>,
  pub no_sources: Option<bool>,
  pub public_path: Option<String>,
  pub source_root: Option<String>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  #[derivative(Debug = "ignore")]
  pub test: Option<RawRules>,
}

impl From<RawSourceMapDevToolPluginOptions> for DevtoolPluginOptions {
  fn from(raw_opts: RawSourceMapDevToolPluginOptions) -> Self {
    let append = raw_opts
      .append
      .map_or(default_append(), |name| normalize_raw_append(name));

    DevtoolPluginOptions {
      append,
      inline: todo!(),
      namespace: todo!(),
      columns: todo!(),
      no_sources: todo!(),
      public_path: raw_opts.public_path,
    }
  }
}
