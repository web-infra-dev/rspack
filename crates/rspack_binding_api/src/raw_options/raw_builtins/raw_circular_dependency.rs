use derive_more::Debug;
use napi::{Either, bindgen_prelude::FnArgs};
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_circular_dependencies::{
  CircularDependencyIgnoredConnection, CircularDependencyIgnoredConnectionEntry,
  CircularDependencyRspackPluginOptions, CompilationHookFn, CycleHandlerFn,
};

use crate::js_regex::JsRegExp;

fn ignore_pattern_to_entry(
  pattern: Either<String, JsRegExp>,
) -> rspack_error::Result<CircularDependencyIgnoredConnectionEntry> {
  Ok(match pattern {
    Either::A(string) => CircularDependencyIgnoredConnectionEntry::String(string),
    Either::B(pattern) => CircularDependencyIgnoredConnectionEntry::Pattern(pattern.try_into()?),
  })
}

type ConnectionPattern = Either<String, JsRegExp>;
type CycleHookParams = (String, Vec<String>);

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCircularDependencyRspackPluginOptions {
  pub fail_on_error: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<JsRegExp>,
  #[napi(ts_type = "Array<[string | RegExp, string | RegExp]>")]
  pub ignored_connections: Option<Vec<(ConnectionPattern, ConnectionPattern)>>,
  #[debug(skip)]
  #[napi(ts_type = "(entrypoint: Module, modules: string[]) => void")]
  pub on_detected: Option<ThreadsafeFunction<FnArgs<CycleHookParams>, ()>>,
  #[debug(skip)]
  #[napi(ts_type = "(entrypoint: Module, modules: string[]) => void")]
  pub on_ignored: Option<ThreadsafeFunction<FnArgs<CycleHookParams>, ()>>,
  #[debug(skip)]
  #[napi(ts_type = "() => void")]
  pub on_start: Option<ThreadsafeFunction<(), ()>>,
  #[debug(skip)]
  #[napi(ts_type = "() => void")]
  pub on_end: Option<ThreadsafeFunction<(), ()>>,
}

impl TryFrom<RawCircularDependencyRspackPluginOptions> for CircularDependencyRspackPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawCircularDependencyRspackPluginOptions) -> Result<Self, Self::Error> {
    // This explicit cast is needed because Rust otherwise infers an incompatible type
    // for the closure compared to the field in the options object.

    let on_detected: Option<CycleHandlerFn> = match value.on_detected {
      Some(callback) => Some(Box::new(move |entrypoint, modules| {
        let callback = callback.clone();
        Box::pin(async move {
          callback
            .call_with_sync((entrypoint, modules).into())
            .await?;
          Ok(())
        })
      })),
      _ => None,
    };
    let on_ignored: Option<CycleHandlerFn> = match value.on_ignored {
      Some(callback) => Some(Box::new(move |entrypoint, modules| {
        Box::pin({
          let callback = callback.clone();
          async move {
            callback
              .call_with_sync((entrypoint, modules).into())
              .await?;
            Ok(())
          }
        })
      })),
      _ => None,
    };
    let on_start: Option<CompilationHookFn> = match value.on_start {
      Some(callback) => Some(Box::new(move || {
        let callback = callback.clone();
        Box::pin({
          async move {
            callback.call_with_sync(()).await?;
            Ok(())
          }
        })
      })),
      _ => None,
    };
    let on_end: Option<CompilationHookFn> = match value.on_end {
      Some(callback) => Some(Box::new(move || {
        let callback = callback.clone();
        Box::pin({
          async move {
            callback.call_with_sync(()).await?;
            Ok(())
          }
        })
      })),
      _ => None,
    };

    Ok(Self {
      fail_on_error: value.fail_on_error.unwrap_or(false),
      exclude: value.exclude.map(TryInto::try_into).transpose()?,
      ignored_connections: value
        .ignored_connections
        .map(|connections: Vec<_>| {
          connections
            .into_iter()
            .map(|(from, to)| {
              Ok(CircularDependencyIgnoredConnection(
                ignore_pattern_to_entry(from)?,
                ignore_pattern_to_entry(to)?,
              ))
            })
            .collect::<Result<_, Self::Error>>()
        })
        .transpose()?,
      on_detected,
      on_ignored,
      on_start,
      on_end,
    })
  }
}
