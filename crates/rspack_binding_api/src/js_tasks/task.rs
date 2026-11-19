use std::sync::Arc;

use napi::{JsValue, bindgen_prelude::*};
use rspack_core::ResolverFactory;
use rspack_error::{Result, error};
use rspack_napi::threadsafe_function::DynThreadsafeFunction;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{
  error::RspackResultToNapiResultExt,
  options::raw_resolve::normalize_raw_resolve_options_with_dependency_type,
  plugins::js_loader::JsLoaderContext, resolver::JsResolver,
};

pub(crate) enum JsTask {
  Loader(LoaderTask),
}

impl JsTask {
  pub(super) async fn run(self, worker: &DynThreadsafeFunction, cancel: CancellationToken) {
    match self {
      Self::Loader(task) => task.run(worker, cancel).await,
    }
  }

  pub(super) fn fail(self, message: &str) {
    match self {
      Self::Loader(task) => task.fail(message),
    }
  }
}

pub(crate) struct LoaderTask {
  pub context: JsLoaderContext,
  pub compiler_handle: u32,
  pub resolver_factory: Option<Arc<ResolverFactory>>,
  pub result: oneshot::Sender<Result<JsLoaderContext>>,
}

impl LoaderTask {
  async fn run(self, worker: &DynThreadsafeFunction, cancel: CancellationToken) {
    let Self {
      context,
      compiler_handle,
      resolver_factory,
      result,
    } = self;
    let outcome = worker
      .call_with_promise_prepared(move |env| {
        if cancel.is_cancelled() {
          return Err(Error::from_reason("JS task subscription has closed"));
        }
        let (context, access) = context.into_jsobject(env)?;
        let mut task = Object::new(env)?;
        task.set("type", "loader")?;
        task.set("context", context)?;
        task.set("compilerHandle", compiler_handle)?;
        if let Some(factory) = resolver_factory {
          let get_resolver: Function<Unknown, JsResolver> =
            env.create_function_from_closure("getResolver", move |call| {
              let options =
                normalize_raw_resolve_options_with_dependency_type(Some(call.get(0)?), false)
                  .to_napi_result()?;
              Ok(JsResolver::new(factory.get(options)))
            })?;
          task.set("getResolver", get_resolver)?;
        }
        Ok((vec![task.raw()], access))
      })
      .await;
    let _ = result.send(outcome);
  }

  fn fail(self, message: &str) {
    let _ = self.result.send(Err(error!("{message}")));
  }
}
