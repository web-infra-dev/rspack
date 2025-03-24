use std::sync::Arc;

use napi::{Either, JsUnknown};
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_circular_dependencies::{
  CircularDependencyIgnoredConnection, CircularDependencyIgnoredConnectionEntry,
  CircularDependencyRspackPluginOptions, CompilationHookFn, CycleHandlerFn,
};
use rspack_regex::RspackRegex;

use crate::JsCompilationWrapper;

fn ignore_pattern_to_entry(
  pattern: Either<String, RspackRegex>,
) -> CircularDependencyIgnoredConnectionEntry {
  match pattern {
    Either::A(string) => CircularDependencyIgnoredConnectionEntry::String(string),
    Either::B(pattern) => CircularDependencyIgnoredConnectionEntry::Pattern(pattern),
  }
}

type ConnectionPattern = Either<String, RspackRegex>;
type CycleHookParams = (String, Vec<String>, JsCompilationWrapper);

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCircularDependencyRspackPluginOptions {
  pub fail_on_error: Option<bool>,
  pub allow_async_cycles: Option<bool>,
  pub exclude: Option<RspackRegex>,
  pub ignored_connections: Option<Vec<(ConnectionPattern, ConnectionPattern)>>,
  #[napi(ts_type = "(entrypoint: Module, modules: string[], compilation: Compilation) => void")]
  pub on_detected: Option<ThreadsafeFunction<CycleHookParams, JsUnknown>>,
  #[napi(ts_type = "(entrypoint: Module, modules: string[], compilation: Compilation) => void")]
  pub on_ignored: Option<ThreadsafeFunction<CycleHookParams, JsUnknown>>,
  #[napi(ts_type = "(compilation: Compilation) => void")]
  pub on_start: Option<ThreadsafeFunction<JsCompilationWrapper, JsUnknown>>,
  #[napi(ts_type = "(compilation: Compilation) => void")]
  pub on_end: Option<ThreadsafeFunction<JsCompilationWrapper, JsUnknown>>,
}

impl From<RawCircularDependencyRspackPluginOptions> for CircularDependencyRspackPluginOptions {
  fn from(value: RawCircularDependencyRspackPluginOptions) -> Self {
    // This explicit cast is needed because Rust otherwise infers an incompatible type
    // for the closure compared to the field in the options object.
    let on_detected: Option<CycleHandlerFn> = match value.on_detected {
      Some(callback) => Some(Arc::new(move |entrypoint, modules, compilation| {
        callback.blocking_call_with_sync((
          entrypoint,
          modules,
          JsCompilationWrapper::new(compilation),
        ))?;
        Ok(())
      })),
      _ => None,
    };
    let on_ignored: Option<CycleHandlerFn> = match value.on_ignored {
      Some(callback) => Some(Arc::new(move |entrypoint, modules, compilation| {
        callback.blocking_call_with_sync((
          entrypoint,
          modules,
          JsCompilationWrapper::new(compilation),
        ))?;
        Ok(())
      })),
      _ => None,
    };
    let on_start: Option<CompilationHookFn> = match value.on_start {
      Some(callback) => Some(Arc::new(move |compilation| {
        callback.blocking_call_with_sync(JsCompilationWrapper::new(compilation))?;
        Ok(())
      })),
      _ => None,
    };
    let on_end: Option<CompilationHookFn> = match value.on_end {
      Some(callback) => Some(Arc::new(move |compilation| {
        callback.blocking_call_with_sync(JsCompilationWrapper::new(compilation))?;
        Ok(())
      })),
      _ => None,
    };

    Self {
      fail_on_error: value.fail_on_error.unwrap_or(false),
      allow_async_cycles: value.allow_async_cycles.unwrap_or(false),
      exclude: value.exclude,
      ignored_connections: value.ignored_connections.map(|connections| {
        connections
          .into_iter()
          .map(|(from, to)| {
            CircularDependencyIgnoredConnection(
              ignore_pattern_to_entry(from),
              ignore_pattern_to_entry(to),
            )
          })
          .collect()
      }),
      on_detected,
      on_ignored,
      on_start,
      on_end,
    }
  }
}
