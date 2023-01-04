use std::fmt::Debug;
use std::pin::Pin;

use napi::{Env, NapiRaw, Result};

use async_trait::async_trait;

use rspack_error::{internal_error, Error};

use crate::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use crate::{JsCompilation, JsHooks};

pub struct JsHooksAdapter {
  // pub done_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub process_assets_tsfn: ThreadsafeFunction<(), ()>,
}

impl Debug for JsHooksAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "rspack_plugin_js_hooks_adapter")
  }
}

#[async_trait]
impl rspack_core::Plugin for JsHooksAdapter {
  fn name(&self) -> &'static str {
    "rspack_plugin_js_hooks_adapter"
  }

  #[tracing::instrument(name = "js_hooks_adapter::compilation", skip_all)]
  async fn compilation(
    &mut self,
    args: rspack_core::CompilationArgs<'_>,
  ) -> rspack_core::PluginCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      Pin::new_unchecked(std::mem::transmute::<
        &'_ mut rspack_core::Compilation,
        &'static mut rspack_core::Compilation,
      >(args.compilation))
    });

    self
      .compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)?
      .await
      .map_err(|err| {
        Error::InternalError(internal_error!(format!(
          "Failed to compilation: {}",
          err.to_string()
        )))
      })?
  }

  #[tracing::instrument(name = "js_hooks_adapter::this_compilation", skip_all)]
  async fn this_compilation(
    &mut self,
    args: rspack_core::ThisCompilationArgs<'_>,
  ) -> rspack_core::PluginThisCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      Pin::new_unchecked(std::mem::transmute::<
        &'_ mut rspack_core::Compilation,
        &'static mut rspack_core::Compilation,
      >(args.this_compilation))
    });

    self
      .this_compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::NonBlocking)?
      .await
      .map_err(|err| {
        Error::InternalError(internal_error!(format!(
          "Failed to this_compilation: {}",
          err.to_string()
        )))
      })?
  }

  #[tracing::instrument(name = "js_hooks_adapter::process_assets", skip_all)]
  async fn process_assets(
    &mut self,
    _ctx: rspack_core::PluginContext,
    _args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsHookOutput {
    // Directly calling hook processAssets without converting assets to JsAssets, instead, we use APIs to get `Source` lazily on the Node side.
    self
      .process_assets_tsfn
      .call((), ThreadsafeFunctionCallMode::NonBlocking)?
      .await
      .map_err(|err| {
        Error::InternalError(internal_error!(format!(
          "Failed to call process assets: {}",
          err.to_string()
        )))
      })?
  }

  // #[tracing::instrument(name = "js_hooks_adapter::done", skip_all)]
  // async fn done<'s, 'c>(
  //   &mut self,
  //   _ctx: rspack_core::PluginContext,
  //   args: rspack_core::DoneArgs<'s, 'c>,
  // ) -> rspack_core::PluginBuildEndHookOutput {
  //   let compilation = JsCompilation::from_compilation(unsafe {
  //     Pin::new_unchecked(std::mem::transmute::<
  //       &'_ mut rspack_core::Compilation,
  //       &'static mut rspack_core::Compilation,
  //     >(args.stats.compilation))
  //   });

  //   self
  //     .done_tsfn
  //     .call(
  //       compilation,
  //       ThreadsafeFunctionCallMode::NonBlocking,
  //     )?
  //     .await
  //     .map_err(|err| {
  //       Error::InternalError(internal_error!(format!(
  //         "Failed to call done: {}",
  //         err.to_string()
  //       )))
  //     })?
  //     .map_err(Error::from)
  // }
}

impl JsHooksAdapter {
  pub fn from_js_hooks(env: Env, js_hooks: JsHooks) -> Result<Self> {
    let JsHooks {
      // done,
      process_assets,
      this_compilation,
      compilation,
    } = js_hooks;

    // *Note* that the order of the creation of threadsafe function is important. There is a queue of threadsafe calls for each tsfn:
    // For example:
    // tsfn1: [call-in-js-task1, call-in-js-task2]
    // tsfn2: [call-in-js-task3, call-in-js-task4]
    // If the tsfn1 is created before tsfn2, and task1 is created(via `tsfn.call`) before task2(single tsfn level),
    // and *if these tasks are created in the same tick*, tasks will be called on main thread in the order of `task1` `task2` `task3` `task4`
    //
    // In practice:
    // The creation of callback `this_compilation` is placed before the callback `compilation` because we want the JS hooks `this_compilation` to be called before the JS hooks `compilation`.

    // let mut done_tsfn: ThreadsafeFunction<JsCompilation, ()> = {
    //   let cb = unsafe { done.raw() };

    //   ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
    //     let (ctx, resolver) = ctx.split_into_parts();

    //     let env = ctx.env;
    //     let cb = ctx.callback;
    //     let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

    //     resolver.resolve::<()>(result, |_| Ok(()))
    //   })
    // }?;

    let mut process_assets_tsfn: ThreadsafeFunction<(), ()> = {
      let cb = unsafe { process_assets.raw() };

      ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
        let (ctx, resolver) = ctx.split_into_parts();

        let env = ctx.env;
        let cb = ctx.callback;
        let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

        resolver.resolve::<()>(result, |_| Ok(()))
      })
    }?;

    let mut this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> = {
      let cb = unsafe { this_compilation.raw() };

      ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
        let (ctx, resolver) = ctx.split_into_parts();

        let env = ctx.env;
        let cb = ctx.callback;
        let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

        resolver.resolve::<()>(result, |_| Ok(()))
      })
    }?;

    let mut compilation_tsfn: ThreadsafeFunction<JsCompilation, ()> = {
      let cb = unsafe { compilation.raw() };

      ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
        let (ctx, resolver) = ctx.split_into_parts();

        let env = ctx.env;
        let cb = ctx.callback;
        let result = unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }?;

        resolver.resolve::<()>(result, |_| Ok(()))
      })
    }?;

    // See the comment in `threadsafe_function.rs`
    // done_tsfn.unref(&env)?;
    process_assets_tsfn.unref(&env)?;
    compilation_tsfn.unref(&env)?;
    this_compilation_tsfn.unref(&env)?;

    Ok(JsHooksAdapter {
      // done_tsfn,
      process_assets_tsfn,
      compilation_tsfn,
      this_compilation_tsfn,
    })
  }
}
