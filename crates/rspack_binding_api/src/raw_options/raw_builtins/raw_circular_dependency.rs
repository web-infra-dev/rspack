use std::{ptr::NonNull, sync::Arc};

use derive_more::Debug;
use futures::future::BoxFuture;
use napi::{Either, bindgen_prelude::FnArgs};
use napi_derive::napi;
use rspack_core::{CompilerId, Module};
use rspack_plugin_circular_dependencies::{
  CircularCheckHandlerFn, CircularCheckRspackPluginOptions, CircularDependencyIgnoredConnection,
  CircularDependencyIgnoredConnectionEntry, CircularDependencyRspackPluginOptions,
  CompilationHookFn, CycleHandlerFn,
};
use rspack_regex::RspackRegex;

use crate::{
  compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction, module::ModuleObject,
};

fn ignore_pattern_to_entry(
  pattern: Either<String, RspackRegex>,
) -> CircularDependencyIgnoredConnectionEntry {
  match pattern {
    Either::A(string) => CircularDependencyIgnoredConnectionEntry::String(string),
    Either::B(pattern) => CircularDependencyIgnoredConnectionEntry::Pattern(pattern),
  }
}

type ConnectionPattern = Either<String, RspackRegex>;
type CycleHookParams = (String, Vec<String>);

/// Deprecated. Use `RawCircularCheckRspackPluginOptions` instead.
#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCircularDependencyRspackPluginOptions {
  pub fail_on_error: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<RspackRegex>,
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

impl From<RawCircularDependencyRspackPluginOptions> for CircularDependencyRspackPluginOptions {
  fn from(value: RawCircularDependencyRspackPluginOptions) -> Self {
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

    Self {
      fail_on_error: value.fail_on_error.unwrap_or(false),
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

type OnDetectedArgs = FnArgs<(ModuleObject, Vec<String>)>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCircularCheckRspackPluginOptions {
  pub fail_on_error: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<RspackRegex>,
  #[napi(ts_type = "RegExp")]
  pub include: Option<RspackRegex>,
  #[debug(skip)]
  #[napi(ts_type = "(module: Module, paths: string[]) => void")]
  pub on_detected: Option<ThreadsafeFunction<OnDetectedArgs, ()>>,
}

impl From<RawCircularCheckRspackPluginOptions> for CircularCheckRspackPluginOptions {
  fn from(value: RawCircularCheckRspackPluginOptions) -> Self {
    let on_detected: Option<CircularCheckHandlerFn> = match value.on_detected {
      Some(callback) => Some(Arc::new(
        move |compiler_id: CompilerId,
              module: &dyn Module,
              paths: Vec<String>|
              -> BoxFuture<'_, rspack_error::Result<()>> {
          let callback = callback.clone();
          let module = ModuleObject::with_ptr(
            NonNull::new(module as *const dyn Module as *mut dyn Module)
              .expect("module pointer should not be null"),
            compiler_id,
          );
          Box::pin(async move {
            callback.call_with_sync((module, paths).into()).await?;
            Ok(())
          })
        },
      )),
      None => None,
    };

    Self {
      exclude: value.exclude,
      include: value.include,
      fail_on_error: value.fail_on_error.unwrap_or(false),
      on_detected,
    }
  }
}
